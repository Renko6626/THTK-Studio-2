use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Box<dyn MasterPty + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

#[derive(Default)]
pub struct PtyManager {
    next_id: AtomicU32,
    sessions: Arc<Mutex<HashMap<u32, PtySession>>>,
}

fn shell_candidates(requested: Option<&str>) -> Vec<String> {
    if let Some(shell) = requested {
        if !shell.trim().is_empty() {
            return vec![shell.trim().to_string()];
        }
    }
    #[cfg(windows)]
    {
        vec![
            "pwsh.exe".to_string(),
            "powershell.exe".to_string(),
            "cmd.exe".to_string(),
        ]
    }
    #[cfg(not(windows))]
    {
        let mut candidates = Vec::new();
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.trim().is_empty() {
                candidates.push(shell);
            }
        }
        candidates.push("bash".to_string());
        candidates.push("sh".to_string());
        candidates
    }
}

impl PtyManager {
    /// Create a new PTY session and return its id.
    ///
    /// # Exit lifecycle contract
    ///
    /// `on_exit` fires **exactly once**, from the waiter thread, after the
    /// session entry has already been removed from the map.  This means:
    ///
    /// * Callers can use the id in `on_exit` for cleanup bookkeeping; it will
    ///   never refer to a live session at that point.
    /// * `kill()` does NOT remove the session — removal happens in the waiter
    ///   thread for both natural exit and kill. `kill()` returns `Err` if the
    ///   session no longer exists (already exited); callers should tolerate that.
    /// * There is no race between natural exit and `kill()`: whichever path
    ///   causes the child to exit, `child.wait()` in the waiter thread
    ///   observes it, removes the entry (dropping `master`, which on Windows
    ///   closes the pseudoconsole and unblocks the reader), and fires
    ///   `on_exit` — once.
    pub fn create(
        &self,
        shell: Option<String>,
        cwd: Option<String>,
        cols: u16,
        rows: u16,
        on_output: impl Fn(u32, Vec<u8>) + Send + 'static,
        on_exit: impl FnOnce(u32, Option<u32>) + Send + 'static,
    ) -> Result<u32, String> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {e}"))?;

        // 依次尝试候选 shell,全部失败才报错
        let mut spawn_error = String::from("No shell candidates");
        let mut child = None;
        for candidate in shell_candidates(shell.as_deref()) {
            let mut cmd = CommandBuilder::new(&candidate);
            if let Some(dir) = cwd.as_deref().filter(|d| !d.trim().is_empty()) {
                cmd.cwd(dir);
            }
            match pair.slave.spawn_command(cmd) {
                Ok(spawned) => {
                    child = Some(spawned);
                    break;
                }
                Err(e) => spawn_error = format!("Failed to spawn '{candidate}': {e}"),
            }
        }
        let mut child = child.ok_or(spawn_error)?;
        drop(pair.slave);

        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let writer = Arc::new(Mutex::new(
            pair.master
                .take_writer()
                .map_err(|e| format!("Failed to open PTY writer: {e}"))?,
        ));
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to open PTY reader: {e}"))?;
        let killer = child.clone_killer();

        // Insert the session BEFORE spawning threads to avoid an orphan-entry
        // race if the shell dies instantly.
        self.sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                id,
                PtySession {
                    writer,
                    master: pair.master,
                    killer,
                },
            );

        // Reader thread: read loop only; never touches child.wait() or on_exit.
        // EINTR (ErrorKind::Interrupted) → continue; EOF (Ok(0)) or other
        // error → break. Dropping this thread after the waiter removes the
        // session (and thus drops `master`) is safe on both Unix and Windows.
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => on_output(id, buf[..n].to_vec()),
                    Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });

        // Waiter thread: owns `child` and a clone of the sessions Arc.
        // Sequence: wait → remove session (drops `master`, closing ConPTY on
        // Windows, unblocking the reader) → fire on_exit.
        let sessions_clone = Arc::clone(&self.sessions);
        std::thread::spawn(move || {
            let code = child.wait().ok().map(|status| status.exit_code());
            // Remove BEFORE calling on_exit so that any code inside on_exit
            // that calls write/kill on this id sees a clean "not found" error.
            sessions_clone
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&id);
            on_exit(id, code);
        });

        Ok(id)
    }

    pub fn write(&self, id: u32, data: &str) -> Result<(), String> {
        // Clone the writer Arc while holding the map lock, then release the map
        // lock before acquiring the per-session writer lock.  This prevents a
        // blocking PTY write (e.g. full input buffer on Ctrl+S or a large paste)
        // from wedging kill/resize/write for ALL other sessions.
        let writer_arc = {
            let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
            sessions
                .get(&id)
                .map(|s| Arc::clone(&s.writer))
                .ok_or_else(|| format!("PTY session {id} not found"))?
        };
        let result = writer_arc
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .write_all(data.as_bytes())
            .map_err(|e| format!("PTY write failed: {e}"));
        result
    }

    pub fn resize(&self, id: u32, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions
            .get(&id)
            .ok_or_else(|| format!("PTY session {id} not found"))?;
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize failed: {e}"))
    }

    /// Signal the child process to exit.
    ///
    /// Does NOT remove the session from the map — the waiter thread does that
    /// (exactly once) and then fires on_exit.  Returns `Err` if the session is
    /// not found (already cleaned up); callers should tolerate that.
    pub fn kill(&self, id: u32) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| format!("PTY session {id} not found"))?;
        let _ = session.killer.kill();
        Ok(())
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn pty_echo_roundtrip_and_exit() {
        let manager = PtyManager::default();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let (exit_tx, exit_rx) = mpsc::channel::<Option<u32>>();

        let id = manager
            .create(
                Some("sh".to_string()),
                None,
                80,
                24,
                move |_id, bytes| {
                    let _ = tx.send(bytes);
                },
                move |_id, code| {
                    let _ = exit_tx.send(code);
                },
            )
            .expect("create pty session");

        manager.write(id, "echo pty_roundtrip_ok\n").expect("write");

        let mut collected = String::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Ok(bytes) = rx.recv_timeout(Duration::from_millis(200)) {
                collected.push_str(&String::from_utf8_lossy(&bytes));
                if collected.contains("pty_roundtrip_ok") {
                    break;
                }
            }
        }
        assert!(
            collected.contains("pty_roundtrip_ok"),
            "PTY output was: {collected}"
        );

        manager.resize(id, 100, 30).expect("resize");
        manager.write(id, "exit\n").expect("write exit");
        exit_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("shell should exit and trigger on_exit");
    }

    #[test]
    fn kill_removes_session() {
        let manager = PtyManager::default();
        let (exit_tx, exit_rx) = mpsc::channel::<Option<u32>>();

        let id = manager
            .create(
                Some("sh".to_string()),
                None,
                80,
                24,
                |_, _| {},
                move |_, code| {
                    let _ = exit_tx.send(code);
                },
            )
            .expect("create");

        manager.kill(id).expect("kill");

        // on_exit fires after the session entry is removed (waiter thread
        // orders: remove → on_exit).  Once we observe on_exit the entry is
        // guaranteed gone — no retry loop needed.
        exit_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("on_exit should fire after kill");

        assert!(
            manager.write(id, "x").is_err(),
            "session should be gone after on_exit"
        );
    }

    #[test]
    fn default_shell_fallback_works() {
        let manager = PtyManager::default();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        let id = manager
            .create(
                None, // use $SHELL / bash / sh fallback
                None,
                80,
                24,
                move |_id, bytes| {
                    let _ = tx.send(bytes);
                },
                |_, _| {},
            )
            .expect("create pty session with default shell");

        manager
            .write(id, "echo fallback_ok\n")
            .expect("write to default shell");

        let mut collected = String::new();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Ok(bytes) = rx.recv_timeout(Duration::from_millis(200)) {
                collected.push_str(&String::from_utf8_lossy(&bytes));
                if collected.contains("fallback_ok") {
                    break;
                }
            }
        }
        assert!(
            collected.contains("fallback_ok"),
            "PTY output from default shell was: {collected}"
        );
    }
}
