# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

THTK-Studio is a **domain-specific desktop IDE for Touhou Project script/asset modding** (ECL / ANM / MSG / STD formats) — not a generic text editor. Architecture, correctness of the modding workflow, and clean frontend/backend separation are prioritized over UI flash. See `project.md` for the long-term roadmap and `AGENTS.md` for behavioral rules.

Stack: Tauri v2 (Rust host) + Vue 3 frontend (currently mostly `.js`, despite the docs' TS goal) + Monaco Editor + Naive UI + Pinia + UnoCSS + Vite.

## Commands

```bash
npm install              # install frontend deps (node_modules is gitignored)
npm run tauri dev        # full desktop app (triggers cargo build; first run is slow)
npm run dev              # frontend only, Vite dev server on 127.0.0.1:1420
npm run build            # frontend build into ./dist
cargo clean --manifest-path src-tauri/Cargo.toml   # reclaim the large target/ dir
```

There is **no frontend test runner configured** (`playwright` is a devDependency but no test script exists) and no linter. "Validation" means checking for type/build issues and import consistency by building.

Rust unit tests exist and run with:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```
15 unit tests cover: pty (3), map_parser (3), mcp_config (3), mcp tools (3), ai_pack (2), auth (1). On Linux the GTK/WebKit dev headers must be visible to compile the Tauri crate — install them via apt, or without sudo use the conda `tauri-dev` environment (exports `PKG_CONFIG_PATH` / `LD_LIBRARY_PATH`); both paths are documented in README "Linux 服务器开发".

`src-tauri/target/` is a Rust debug build cache and grows to several GB (the `windows` crate + incremental cache dominate); it and `node_modules/`, `dist/` are gitignored and regenerated locally.

## Layer model — the central architectural rule

Every change belongs to exactly one layer; **do not mix them casually** (per `AGENTS.md`):

1. **Rust host (`src-tauri/src/`)** — owns everything "system / toolchain / performance": filesystem, workspace scanning, external tool invocation, parsing/indexing, config, file watching, encoding. Heavy logic lives here, not in the frontend.
2. **Tauri commands** — the *boundary only*. `main.rs` does entry + `invoke_handler` registration and nothing else; new logic goes into `common/` or `modules/`, never back into `main.rs`. Commands return **structured results** (e.g. `{ success, stdout, stderr, exit_code, cwd, diagnostics }`) so the frontend never re-parses raw text.
3. **Frontend (`src/`)** — UI only: workbench layout, file tree, tabs, panels, Monaco editor, visualizing the structured data Rust returns.

Before implementing, identify whether the task is a frontend view/panel, editor integration, Rust command/service, toolchain wrapper, or parser/symbol/preview pipeline.

## Backend structure (`src-tauri/src/`)

- `main.rs` — entry + command registration (see the `invoke_handler!` list for the full API surface).
- `app_state.rs` — global `AppState`: `ConfigManager`, current project root, file watcher handle.
- `config.rs` — app config read/write (still uses `Result<_, String>` + some unwrap; MVP-era).
- `common/` — shared capabilities: `fs_utils` (lazy shallow file-tree scan + `get_dir_children` on-demand), `fs_ops` (file CRUD), `file_watcher` (notify + debounce → Tauri event emit), `cmd_runner` (external process exec with Shift-JIS decode + hidden console on Windows), `toolchain` (tool path resolution + version detection), `terminal` (one-shot shell exec — **superseded by PTY terminal, pending cleanup**), `system_clipboard`, `project_config`, `pty` (PTY session management: portable-pty, cross-platform shell detection, ConPTY-safe waiter-thread exit detection, 16ms output batching), `mcp_config` (non-destructive `.mcp.json` merge — preserves existing entries, updates thtk-studio port/token on each launch).
- `modules/ecl/` — the one fully-wired toolchain: `commands` (Tauri commands), `compiler` (builds `thecl` args, runs it), `error_parser` (thecl stderr → `Diagnostic` with normalized absolute paths), `map_parser` (eclmap parsing + global register parsing via `!gvar_names`/`!gvar_types`). New toolchains (thmsg/thstd/thanm) should follow this module shape and reuse the structured-result pattern.
- `modules/mcp/` — in-process MCP server (rmcp 1.7, Streamable HTTP, random port on 127.0.0.1, Bearer token rotated each launch). Six tools: `get_workspace_info`, `check_ecl`, `compile_ecl`, `decompile_ecl`, `lookup_ecl_semantics`, `report_to_user`. Blocking work runs via `spawn_blocking`.

**Encoding matters**: `save_file`'s `is_source` flag decides UTF-8 (`.decl`/`.dmsg` source) vs Shift-JIS (raw game text). Don't lose this distinction.

## Frontend structure (`src/`)

- `api/index.js` — single bridge from frontend to Tauri `invoke` commands.
- `stores/` (Pinia) — keep **domain state separate from UI state**: `project` (workspace/tree), `editor` (tabs/active/dirty/session), `terminal`, `explorerView`/`workbenchPanels` (pure UI selection & visibility), `buildDialog`, `toolchainSettings`, `workbenchReports`.
- `composables/` — complex behavior is extracted here, not left in SFCs (e.g. `useWorkbenchSession`, `useWorkbenchShortcuts`, `useBeforeUnloadGuard`, `useFileWatcher`, `useTheclActions`, `useFileTree{DnD,Actions}`, `useMcpBridge` — listens for `report_to_user` MCP tool calls and publishes structured cards to the output panel). New complex behavior follows this pattern; `App.vue` and `FileTree.vue` are the components most prone to bloat — push logic down.
- `services/` — domain logic that is legitimately frontend-side:
  - `services/toolchains/registry.js` — **registry-driven extensibility point**. `TOOLCHAIN_REGISTRY` maps tool id → descriptor (label, exe name, build-dialog form, request builder, executor). `thecl` is fully implemented; `thmsg`/`thanm`/`thstd`/`thdat` are registered stubs (`supportsBuildDialog: false`) waiting for wiring. Add a new tool by adding a descriptor here.
  - `services/workbench/editorViews.js` — `WORKBENCH_EDITOR_VIEWS` registry mapping view type → editor component (`text` → Monaco, `binary-script` → BinaryScriptView). New file-type workspaces register a view here.
  - `services/languages/ecl/` — the ECL language service: Monaco providers (completion, hover, definition, references, signature help, document symbols), `tokenizer`/`theme` for highlighting, `static-diagnostics` + `toolchain-diagnostics` (thecl output → Monaco markers), and `semantic-loader`/`vocabulary`/`dynamic-vocabulary` which load **eclmap semantic data** to drive the providers. `register.js` wires it all into Monaco.
  - `services/terminal/sessionRuntime.js` — module-level xterm.js runtime; holds `Terminal` instances outside the Vue component tree so sessions survive component unmount/remount. `TerminalPanel.vue` is the multi-tab shell UI (scrollback 5000 lines).

## Agent channel

Running `claude` (claude code) inside the built-in terminal automatically discovers the thtk-studio MCP server: the project root `.mcp.json` is non-destructively maintained by the Rust backend on each project open (port and Bearer token updated; existing custom entries preserved). From inside claude, `/mcp` lists six tools covering workspace info, ECL compile/decompile/check, semantic lookup, and reporting.

The "生成 AI 辅助包" menu action writes `.claude/skills/ecl-modding/` (SKILL.md created once and not overwritten; `references/` regenerated from the current eclmap). This gives claude code domain vocabulary and workflow guidance for ECL modding without manual setup.

## Toolchain integration notes

External Touhou tools (`thecl`, `thmsg`, `thanm`, `thstd`, `thdat`) are wrapped behind a stable internal interface; never hardcode paths/versions/args. Tool paths are **configurable in app settings** (`thecl_path` / `thtk_dir`). The bundled `tools/*.exe` are **Windows binaries** — on Linux, install the Linux build of [thtk](https://github.com/thpatch/thtk) and point the config at it; no source changes needed (see README "Linux 服务器开发"). Do not break compatibility with these external tools unless explicitly asked.
