// Tauri v2 保留的 library 入口（用于移动端构建）。
// 桌面端实际入口在 main.rs。
// 当前项目仅面向 Windows 桌面，此文件保持最小存根即可。

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
