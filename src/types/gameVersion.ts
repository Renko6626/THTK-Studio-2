/**
 * 游戏版本条目。对应 Rust 侧 `common/game_version_commands.rs` 的
 * `GameVersionView`（serde `rename_all = "camelCase"`）。
 *
 * `tools` 里的 id 与 `ToolchainStatus.tool` 对齐：thecl / thanm / thstd / thmsg / thdat。
 * 注意各工具支持的版本集合**并不相同**——thmsg 没有 103（Uwabami Breakers），
 * thdat 多出 PC-98 五作与三部格斗作。不要拿一张列表通吃五个工具。
 */
export interface GameVersionView {
  /** thtk 命令行接受的数字，例如 18。thtk 按 %u 解析，"th18" 这类写法它不认 */
  id: number
  /** 规范化写法 th18，用于文件名匹配与展示 */
  code: string
  title: string
  tools: string[]
}
