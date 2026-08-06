# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

THTK-Studio is a **domain-specific desktop IDE for Touhou Project script/asset modding** (ECL / ANM / MSG / STD formats) — not a generic text editor. Architecture, correctness of the modding workflow, and clean frontend/backend separation are prioritized over UI flash. See `project.md` for the long-term roadmap and `AGENTS.md` for behavioral rules.

Stack: Tauri v2 (Rust host) + Vue 3 + TypeScript frontend + Monaco Editor + Naive UI + Pinia + UnoCSS + Vite.

## Commands

```bash
npm install              # install frontend deps (node_modules is gitignored)
npm run tauri dev        # full desktop app (triggers cargo build; first run is slow)
npm run dev              # frontend only, Vite dev server on 127.0.0.1:1420
npm run build            # frontend build into ./dist
npm test                 # vitest unit tests (domain logic only — see below)
npm run typecheck        # vue-tsc over the .ts boundaries
cargo clean --manifest-path src-tauri/Cargo.toml   # reclaim the large target/ dir
```

Frontend gates (added 2026-07-28):

```bash
npm test          # vitest run — 45 unit tests over stores / composables / services
npm run typecheck # vue-tsc over all .ts and lang="ts" SFCs
```

Scope is deliberate: `vitest.config.js` covers **domain logic**, not component rendering — component tests would need either the full naive-ui provider chain or a real Tauri host, and no Playwright/e2e exists because driving the Tauri window needs a display this dev box doesn't have. UI behaviour is verified by the Windows manual checklist in the MVP closure plan.

**The frontend is fully TypeScript** — `allowJs: false`, and `src/` plus `tests/` contain zero `.js`. Every SFC carries `lang="ts"` (except `EmptyEditorState.vue`, which is template-only). Adding a `.js` file will fail typecheck with "cannot find module", which is the intended guard rail.

Cross-boundary types live in `src/types/` and are written field-by-field against the Rust structs. **The backend's serde naming is not uniform** — `ProjectConfig`/`RecentProject` are camelCase, `AppConfig` is snake_case, `FileNode` is mixed (`is_dir` but `isLeaf`), and `EclMapInstructionParameter.type_name` is renamed to `type`. `src-tauri/src/wire_format_tests.rs` pins the serialized key sets so a Rust struct change fails there instead of silently breaking the frontend at runtime.

`typescript` must stay pinned to 5.x (7.x dropped the `./lib/tsc` export `vue-tsc` needs). There is no linter.

Rust unit tests exist and run with:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```
132 unit tests cover: mcp_config (20), project_config (12), fs_ops (12), wire_format (9), msg/translator (9), thstd/translator (9), recent_projects (8), msg/compiler (7), thdat/compiler (6), fs_utils (6), mcp/tools (5), thstd/compiler (4), pty (4), config (3), toolchain (3), ecl/map_parser (3), ecl/ai_pack (3), mcp/server (3), msg/map_parser (3), thstd/map_parser (3). On Linux the GTK/WebKit dev headers must be visible to compile the Tauri crate — install them via apt, or without sudo use the conda `tauri-dev` environment (exports `PKG_CONFIG_PATH` / `LD_LIBRARY_PATH`); both paths are documented in README "Linux 服务器开发".

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
- `common/` — shared capabilities: `fs_utils` (lazy shallow file-tree scan + `get_dir_children` on-demand + `validate_project_dir`), `fs_ops` (file CRUD), `file_watcher` (notify + debounce → Tauri event emit), `cmd_runner` (external process exec with Shift-JIS decode + hidden console on Windows), `toolchain` (tool path resolution + version detection + `effective_config` applying the project-level `thtkDir` override), `system_clipboard`, `project_config` (three-state load — absent/loaded/invalid — plus atomic temp+rename save), `recent_projects` (dedupe/promote/cap-10, path normalization mirroring the frontend's `pathNormalize.js`), `pty` (PTY session management: portable-pty, cross-platform shell detection, ConPTY-safe waiter-thread exit detection, 16ms output batching), `mcp_config` (non-destructive `.mcp.json` merge — preserves existing entries, updates thtk-studio port/token on each launch).
- `modules/ecl/` — the one fully-wired toolchain: `commands` (Tauri commands), `compiler` (builds `thecl` args, runs it), `error_parser` (thecl stderr → `Diagnostic` with normalized absolute paths), `map_parser` (eclmap parsing + global register parsing via `!gvar_names`/`!gvar_types`). New toolchains (thmsg/thstd/thanm) should follow this module shape and reuse the structured-result pattern.
- `modules/msg/` — second toolchain (basic workflow only): `compiler` (thmsg -d/-c with SJIS↔UTF-8 bridge), `translator` (line-level `ins_N` ↔ name using the JSON semantic data; preserves time labels and unknown opcodes), `map_parser` (loads embedded `assets/msg-th{ver}.json`, currently only th17 with fallback), `commands`. No Monaco language service, no MCP tools, no AI assist pack — see the spec's "不做花活" section.
- `modules/thstd/` — third toolchain (basic workflow only): `compiler` (thstd -d/-c, fully UTF-8 — thstd files have no Japanese text, so no SJIS bridge needed), `translator` (line-level `ins_N` ↔ name with special handling for opcode 1 (jmp) — thstd binary uses `ins_1(offset, time)` while ref/ECL ecosystem uses `jmp(time, offset)`, so the translator swaps args on both directions), `map_parser` (loads embedded `assets/std-th17.json`, 19 instructions, falls back across versions), `commands`. Same scope discipline as `modules/msg/` — no Monaco language service, no MCP tools, no AI assist pack.
- `modules/thdat/` — fourth toolchain (container manager): `compiler` (thdat -xd extract with auto-detect / -c{ver} pack via `cmd_runner::run_tool`), `commands` (`extract_dat_file` / `pack_dat_file`). **No `map_parser` or `translator`** — thdat is a packing/unpacking tool with no script semantics. Pack guards against the Windows 32KB command-line limit by precomputing argument byte length and erroring above 28KB (.dat archives can contain hundreds of files; batching not implemented this round).
- `modules/mcp/` — in-process MCP server (rmcp 1.7, Streamable HTTP, random port on 127.0.0.1, Bearer token rotated each launch). Six tools: `get_workspace_info`, `check_ecl`, `compile_ecl`, `decompile_ecl`, `lookup_ecl_semantics`, `report_to_user`. Blocking work runs via `spawn_blocking`.

### Why MSG / STD have a translator but ECL / ANM don't

thtk's `-m` mapfile support is **asymmetric**: `thecl` and `thanm` have it, `thmsg` and
`thstd` do not. So ECL/ANM instruction naming is done by the tool itself (we just pass
`-m`), while MSG/STD naming is done by our own `translator` layer sitting outside the tool.

The consequence is that **`.dmsg` / `.dstd` on disk are an IDE dialect, not valid thmsg /
thstd input** — those tools only ever see `ins_N` (we translate into a temp file before
invoking them). Running `thmsg -c 20 file.dmsg out.msg` by hand fails on every named
instruction. This is a deliberate trade: `git diff` readability of `textboxShow(0)` over
`ins_3(0)` is worth it, but it must be declared in the file and an export path provided.

`.decl` has no such problem — thecl understands eclmap names natively.

Full decision record with the evidence (thtk source version tables, truth's coverage,
thcrap's Unicode statement): `docs/toolchain-ecosystem.md`.

**Encoding matters**: `save_file`'s `is_source` flag decides UTF-8 (`.decl`/`.dmsg` source) vs Shift-JIS (raw game text). Don't lose this distinction.

## Frontend structure (`src/`)

- `api/index.js` — single bridge from frontend to Tauri `invoke` commands.
- `stores/` (Pinia) — keep **domain state separate from UI state**: `project` (workspace/tree/project-config tri-state), `recentProjects`, `editor` (tabs/active/dirty/session), `terminal`, `explorerView`/`workbenchPanels` (pure UI selection & visibility), `buildDialog`, `toolchainSettings`, `projectSettings`, `workbenchReports`.
- `composables/` — complex behavior is extracted here, not left in SFCs (e.g. `useWorkbenchSession`, `useWorkbenchShortcuts`, `useBeforeUnloadGuard`, `useFileWatcher`, `useTheclActions`, `useFileTree{DnD,Actions}`, `useMcpBridge` — listens for `report_to_user` MCP tool calls and publishes structured cards to the output panel). New complex behavior follows this pattern; `App.vue` and `FileTree.vue` are the components most prone to bloat — push logic down.
  - `useToolchainActions` / `useToolchainResult` — toolchain action dispatcher and unified result publisher; used by both MenuBar and BinaryScriptView so the two stay in sync.
  - `useProjectActions` — **the only** way to open/switch projects. Menu, Ctrl+O, file tree and the welcome page all route through it; it owns the dirty-tab three-way confirm, post-switch cleanup and error reporting. Do not call `projectStore.loadProject` directly from a component.

`App.vue` holds **only** the naive-ui providers; the workbench lives in `components/Layout/WorkbenchRoot.vue`. This is deliberate — `useMessage`/`useDialog` only resolve inside a provider's descendants, so workbench-level composables that need toasts or confirms cannot live in `App.vue`'s own setup.
- `services/` — domain logic that is legitimately frontend-side:
  - `services/toolchains/registry.js` — **registry-driven extensibility point**. `TOOLCHAIN_REGISTRY` maps tool id → descriptor (label, exe name, build-dialog form, request builder, executor). `thecl` is fully implemented; `thmsg`/`thanm`/`thstd`/`thdat` are registered stubs (`supportsBuildDialog: false`) waiting for wiring. Add a new tool by adding a descriptor here.
  - `services/workbench/editorViews.js` — `WORKBENCH_EDITOR_VIEWS` registry mapping view type → editor component (`text` → Monaco, `binary-script` → BinaryScriptView). New file-type workspaces register a view here. `WorkbenchEditorHost` falls back to `Welcome/WelcomeView.vue` when there is no workspace, and to the lightweight `EmptyEditorState` when a workspace is open but no tab is active.
  - `services/languages/ecl/` — the ECL language service: Monaco providers (completion, hover, definition, references, signature help, document symbols), `tokenizer`/`theme` for highlighting, `static-diagnostics` + `toolchain-diagnostics` (thecl output → Monaco markers), and `semantic-loader`/`vocabulary`/`dynamic-vocabulary` which load **eclmap semantic data** to drive the providers. `register.js` wires it all into Monaco.
  - `services/terminal/sessionRuntime.js` — module-level xterm.js runtime; holds `Terminal` instances outside the Vue component tree so sessions survive component unmount/remount. `TerminalPanel.vue` is the multi-tab shell UI (scrollback 5000 lines).

## Agent channel

Running `claude` (claude code) inside the built-in terminal automatically discovers the thtk-studio MCP server: the project root `.mcp.json` is non-destructively maintained by the Rust backend on each project open (port and Bearer token updated; existing custom entries preserved). From inside claude, `/mcp` lists six tools covering workspace info, ECL compile/decompile/check, semantic lookup, and reporting.

codex and opencode are also auto-registered: on project open the IDE detects whether the corresponding CLI is on `PATH` and, if so, manages the project-level `.codex/config.toml` (codex requires a one-time trust in the project directory) and `opencode.json`. PTY sessions are injected with `THTK_MCP_URL` and `THTK_MCP_TOKEN` so any tool can reference them as env vars or self-register via the SKILL.md guide. The MCP port defaults to a fixed 39127 (`mcp_port` config key; falls back to a random ephemeral port on conflict).

The "生成 AI 辅助包" menu action writes `.claude/skills/ecl-modding/` (SKILL.md created once and not overwritten; `references/` regenerated from the current eclmap). This gives claude code domain vocabulary and workflow guidance for ECL modding without manual setup.

## Toolchain integration notes

External Touhou tools (`thecl`, `thmsg`, `thanm`, `thstd`, `thdat`) are wrapped behind a stable internal interface; never hardcode paths/versions/args. Tool paths are **configurable in app settings** (`thecl_path` / `thtk_dir`). The bundled `tools/*.exe` are **Windows binaries** — on Linux, install the Linux build of [thtk](https://github.com/thpatch/thtk) and point the config at it; no source changes needed (see README "Linux 服务器开发"). Do not break compatibility with these external tools unless explicitly asked.
