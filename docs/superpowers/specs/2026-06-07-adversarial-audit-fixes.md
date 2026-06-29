# 对抗审阅修复批次(2026-06-07)

日期:2026-06-07 状态:已落地
来源:外部知识对抗审阅(WebSearch + WebFetch 调研 + 代码对照)
前置:Batch 1 + Batch 2 已完成,本批是审阅暴露的"修复尚未覆盖到的角落"

## 触发

Batch 1+2 修完后,以 deep-research 风格做对抗审阅:用外部 CVE / Tauri 官方文档 /
notify-rs 文档等已知陷阱对照我们的实现。结果:Batch 1+2 把 file-tree 子系统
本身收得很干净,但暴露了**周边老代码的相同问题**——具体是 main.rs 里两个 MVP
时代的命令和 tauri.conf.json 的 CSP 配置。

## 修复清单

### Critical

1. **read_file / save_file 缺路径守卫** — main.rs:63-78 两个 MVP 时代的命令
   从未接 guard_path。Batch 1 把 fs_ops 5 个写命令全套上守卫,但这俩老命令是
   旁路。结果:webview XSS 或恶意 MCP 工具可以 `read_file("/etc/passwd")` 直接拿到
   内容,所有 file-tree hardening 形同虚设。
   **已修复(commit 2415baa)**:`guard_path` 升 pub(crate),read_file/save_file
   接 `State<AppState>` 参数,前置校验。审计所有 frontend 调用方,无项目外合法用例。

2. **CSP = null,XSS = RCE** — tauri.conf.json `security.csp` 是 null。任何 webview
   XSS 都能调用 `pty_create` / `pty_write` / `run_shell_command`,等于任意命令执行。
   **已修复(commit 4e1ba7e)**:严格 CSP(无 script-src unsafe-inline/eval),
   允许 ipc: / 127.0.0.1:* (MCP server) / blob: (Monaco workers) / data: (图片)。
   index.html 干净无 inline script;frontend 全无 innerHTML / document.write 用法。

### Important

3. **后端不拒 `:` / Windows 保留名** — frontend `validateFileName` 有,但
   MCP/agent 直调 backend 能植入 NTFS ADS(`file.txt:hidden`)或设备名(`NUL.txt`)。
   **已修复(commit 2415baa)**:`fs_ops::validate_basename` 镜像前端规则,
   create_file / create_directory / rename_entry 都调用。6 个新单测覆盖
   分隔符 / 冒号 / 保留名 / 控制字符 / 点点 / 正常名。

4. **`copy_dir_all` 只拦 symlink,不拦 Windows junction** — `is_symlink()` 返回 false 时
   `mklink /J` 的 junction reparse point 滑过去,递归复制可能跑出工作区。
   **已修复(commit 2415baa)**:`#[cfg(windows)]` 加 `is_reparse_point` helper,
   检 FILE_ATTRIBUTE_REPARSE_POINT (0x400),`is_symlink() || is_reparse_point` 任一成立即拒。

### Minor / 文档

5. **`stat_entry` 故意不守卫是设计权衡** — 系统剪贴板需要探测 root 外路径。
   副作用:XSS 能用它当主机路径"存在性 oracle"。
   **已修复(commit 2415baa)**:加详细 SECURITY 注释,说明威胁模型与可接受性。

## 不在范围(Batch 3 候选)

- rename TOCTOU on Linux(单用户 IDE,academic)
- Watcher mid-flight 失败回路(只在启动期可见)
- `remapExpandedKeys` 混合分隔符(UX,非安全)
- `current_project_root` 没预 canonicalize(性能,非安全)
- APFS NFC/NFD UX 状态不一致(无 macOS 用户)

## 收益

- 测试 88 → 94(+6 validate_basename)
- 主线 IDE 攻击面收得相当紧:webview XSS 不再 = 任意文件读 / RCE
- 周边老命令的"hardening 盲点"被堵上,security posture 与 file-tree 一致
