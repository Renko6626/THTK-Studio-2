use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub struct PtySession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

#[derive(Default)]
pub struct PtyManager {
    next_id: AtomicU32,
    sessions: Mutex<HashMap<u32, PtySession>>,
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
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to open PTY writer: {e}"))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to open PTY reader: {e}"))?;
        let killer = child.clone_killer();

        // 每会话一个 reader 线程:读到 EOF 后等待子进程退出码并回调 on_exit
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => on_output(id, buf[..n].to_vec()),
                }
            }
            let code = child.wait().ok().map(|status| status.exit_code());
            on_exit(id, code);
        });

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
        Ok(id)
    }

    pub fn write(&self, id: u32, data: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| format!("PTY session {id} not found"))?;
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("PTY write failed: {e}"))
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

    /// 杀进程并移除会话。进程可能已自然退出,kill 失败不视为错误。
    pub fn kill(&self, id: u32) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let mut session = sessions
            .remove(&id)
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
        let id = manager
            .create(Some("sh".to_string()), None, 80, 24, |_, _| {}, |_, _| {})
            .expect("create");
        manager.kill(id).expect("kill");
        assert!(manager.write(id, "x").is_err(), "session should be gone");
    }
}
