# 终端 shell 选择器(最小版)

日期:2026-06-07 状态:已批准

- 终端面板 ➕ 旁加下拉(n-dropdown):按平台列出常用 shell
  (Windows: pwsh / powershell / cmd / Git Bash;Unix: $SHELL 默认 / bash / zsh / fish)。
  平台用 navigator.userAgent 判断。➕ 本身仍走后端自动探测,行为不变。
- store `openSession({ shell, label })`:指定 shell 启动失败时**回退默认探测**并在输出面板
  报一张提示卡;回退也失败才走既有的失败卡片。tab 标题带 shell 名,如 `终端 2 (cmd)`。
- 不做:settings 里的 profile 配置、自定义 shell 路径(后续有需要再加)。
- 验证:npm run build + Windows 手动验收(各 shell 打开、选一个不存在的看回退)。
