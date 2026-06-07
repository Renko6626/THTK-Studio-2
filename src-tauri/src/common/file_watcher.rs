use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: String, // "modify" | "remove" | "create"
}

pub struct FileWatcherState {
    /// 持有 debouncer 的所有权，drop 时自动停止监听
    _debouncer: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
}

impl FileWatcherState {
    pub fn new() -> Self {
        Self { _debouncer: None }
    }
}

/// 启动或重置文件监听器，监听指定目录的变化
pub fn start_watching(
    watcher_state: &Mutex<FileWatcherState>,
    app_handle: &AppHandle,
    root_path: &str,
) {
    let mut state = watcher_state.lock().unwrap_or_else(|e| e.into_inner());

    // Drop 旧的 watcher（自动停止监听）
    state._debouncer = None;

    let app_handle = app_handle.clone();
    let root = PathBuf::from(root_path);

    let debouncer = new_debouncer(
        Duration::from_millis(500),
        move |result: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            let events = match result {
                Ok(events) => events,
                Err(err) => {
                    eprintln!("[file-watcher] error: {:?}", err);
                    return;
                }
            };

            let mut changes: Vec<FileChangeEvent> = Vec::new();
            let mut seen = std::collections::HashSet::new();

            for event in events {
                let path_str = event.path.to_string_lossy().to_string();

                // 跳过目录事件和重复路径
                if event.path.is_dir() || !seen.insert(path_str.clone()) {
                    continue;
                }

                let kind = match event.kind {
                    DebouncedEventKind::Any => {
                        if event.path.exists() {
                            "modify"
                        } else {
                            "remove"
                        }
                    }
                    _ => continue,
                };

                changes.push(FileChangeEvent {
                    path: path_str,
                    kind: kind.to_string(),
                });
            }

            if !changes.is_empty() {
                let _ = app_handle.emit("file-system-changed", &changes);
            }
        },
    );

    match debouncer {
        Ok(mut debouncer) => {
            if let Err(err) = debouncer.watcher().watch(&root, RecursiveMode::Recursive) {
                eprintln!("[file-watcher] failed to watch {:?}: {:?}", root, err);
                return;
            }
            state._debouncer = Some(debouncer);
        }
        Err(err) => {
            eprintln!("[file-watcher] failed to create debouncer: {:?}", err);
        }
    }
}

/// 停止文件监听
pub fn stop_watching(watcher_state: &Mutex<FileWatcherState>) {
    let mut state = watcher_state.lock().unwrap_or_else(|e| e.into_inner());
    state._debouncer = None;
}
