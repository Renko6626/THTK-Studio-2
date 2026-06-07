use crate::common::file_watcher::FileWatcherState;
use crate::config::ConfigManager;
use std::sync::Mutex;

pub struct AppState {
    // 配置管理器 (内含 Mutex)
    pub config_manager: ConfigManager,

    // 当前打开的项目根路径 (用于文件树扫描、注入基准路径)
    // 使用 Option 因为刚启动时可能没打开任何文件夹
    pub current_project_root: Mutex<Option<String>>,

    // 文件系统变更监听器
    pub file_watcher: Mutex<FileWatcherState>,

    // PTY 终端会话管理 (内含 Mutex)
    pub pty_manager: crate::common::pty::PtyManager,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config_manager: ConfigManager::new(),
            current_project_root: Mutex::new(None),
            file_watcher: Mutex::new(FileWatcherState::new()),
            pty_manager: crate::common::pty::PtyManager::default(),
        }
    }
}
