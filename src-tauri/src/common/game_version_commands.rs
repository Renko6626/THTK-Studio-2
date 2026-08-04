use serde::Serialize;

/// 传给前端的版本条目。与 `game_version::GameVersionInfo` 一一对应，
/// 只是把 `&'static str` 换成 owned，并显式声明 camelCase。
///
/// 前端据此渲染下拉框并判断某个工具在当前版本下是否可用——各工具支持的
/// 版本集合并不相同，不能用一张列表通吃。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersionView {
    pub id: u32,
    pub code: String,
    pub title: String,
    pub tools: Vec<String>,
}

#[tauri::command]
pub fn list_game_versions() -> Vec<GameVersionView> {
    crate::common::game_version::GAME_VERSIONS
        .iter()
        .map(|info| GameVersionView {
            id: info.id,
            code: info.code.to_string(),
            title: info.title.to_string(),
            tools: info.tools.iter().map(|t| t.to_string()).collect(),
        })
        .collect()
}
