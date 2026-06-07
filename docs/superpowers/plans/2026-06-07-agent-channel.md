# Agent 通道实现计划(PTY 终端 + MCP server + AI 辅助包)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 AI agent(claude code 等)成为 THTK-Studio 的一等公民:真 PTY 终端 + 进程内 MCP server(六工具)+ 从 eclmap 生成项目级 AI skill。

**Architecture:** Rust 侧新增 `common/pty.rs`(portable-pty 会话管理,输出经 Tauri 事件流式推送)与 `modules/mcp/`(rmcp Streamable HTTP server,绑 127.0.0.1 随机端口 + bearer token,工具直接调用现有 ecl 模块);打开项目时非破坏性合并 `.mcp.json`。前端用 @xterm/xterm 重写终端面板,xterm 实例存活在模块级运行时(脱离组件生命周期)。

**Tech Stack:** Rust: portable-pty 0.9 / rmcp 1.7 / axum 0.8 / tokio / uuid。前端: @xterm/xterm + @xterm/addon-fit。

**Spec:** `docs/superpowers/specs/2026-06-07-agent-channel-design.md`

**约定:** 所有 Rust 测试命令在仓库根执行:`cargo test --manifest-path src-tauri/Cargo.toml <filter>`。前端验证 = `npm run build` 成功。本机(Linux)无法手动跑完整 Tauri 桌面验证,PTY 单测覆盖 Unix 路径,Windows 路径靠代码评审 + 用户机器验收。

---

### Task 1: 添加依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`(经 npm install)

- [ ] **Step 1: Rust 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 末尾(`notify-debouncer-mini` 行后)追加:

```toml
portable-pty = "0.9"   # 跨平台 PTY (Windows ConPTY / Unix openpty)
rmcp = { version = "1.7", features = ["server", "macros", "transport-streamable-http-server"] }
axum = "0.8"
tokio = { version = "1", features = ["net", "rt-multi-thread", "macros", "time"] }
uuid = { version = "1", features = ["v4"] }
```

并新增段落:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: 验证 Rust 依赖可解析**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过(首次下载较慢)。若 rmcp feature 名报错,运行 `cargo add rmcp --manifest-path src-tauri/Cargo.toml --dry-run` 查看可用 features 并修正(1.7.0 的 server/macros/transport-streamable-http-server 来自官方示例)。

- [ ] **Step 3: 前端依赖**

Run: `npm install @xterm/xterm @xterm/addon-fit`
Expected: package.json dependencies 出现两个包,无 peer 冲突。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock package.json package-lock.json
git commit -m "chore: add pty/mcp/xterm dependencies for agent channel"
```

---

### Task 2: PTY 会话管理器(TDD)

**Files:**
- Create: `src-tauri/src/common/pty.rs`
- Modify: `src-tauri/src/common/mod.rs`(追加 `pub mod pty;`)

设计要点:`PtyManager` 与 Tauri 解耦——输出/退出通过调用方传入的闭包回调,单测无需 Tauri 运行时。

- [ ] **Step 1: 写失败测试**

创建 `src-tauri/src/common/pty.rs`,先只写骨架 + 测试(实现留空会编译失败,这就是"失败测试"形态):

```rust
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
```

在 `src-tauri/src/common/mod.rs` 追加一行 `pub mod pty;`。

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml pty`
Expected: 编译错误 `no method named 'create' found`(方法未实现)。

- [ ] **Step 3: 实现 PtyManager**

在 `PtyManager` 定义后、`#[cfg(test)]` 前补全实现:

```rust
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
```

- [ ] **Step 4: 运行确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml pty`
Expected: `test result: ok. 2 passed`(Unix 上)。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/common/pty.rs src-tauri/src/common/mod.rs
git commit -m "feat: PTY session manager with portable-pty (tested echo roundtrip)"
```

---

### Task 3: PTY Tauri 命令 + AppState 挂载

**Files:**
- Modify: `src-tauri/src/common/pty.rs`(文件末尾追加命令)
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src-tauri/src/main.rs:139-165`(invoke_handler)

- [ ] **Step 1: AppState 增加 pty_manager**

`src-tauri/src/app_state.rs` — `file_watcher` 字段后追加字段,`new()` 内同步初始化:

```rust
    // PTY 终端会话管理 (内含 Mutex)
    pub pty_manager: crate::common::pty::PtyManager,
```

`new()` 中:

```rust
            pty_manager: crate::common::pty::PtyManager::default(),
```

- [ ] **Step 2: 在 pty.rs 末尾(tests 模块前)追加 Tauri 命令**

```rust
use crate::app_state::AppState;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyExitPayload {
    pub code: Option<u32>,
}

#[tauri::command]
pub fn pty_create(
    app: AppHandle,
    state: State<AppState>,
    shell: Option<String>,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<u32, String> {
    // 默认工作目录 = 当前项目根
    let cwd = cwd.filter(|c| !c.trim().is_empty()).or_else(|| {
        state
            .current_project_root
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    });

    let out_app = app.clone();
    let exit_app = app;
    state.pty_manager.create(
        shell,
        cwd,
        cols,
        rows,
        move |id, bytes| {
            // 注:lossy 转换在 UTF-8 多字节序列跨 chunk 边界时可能产生替换字符,
            // 实际终端输出以行为主,可接受;后续如有问题改为累积解码。
            let _ = out_app.emit(
                &format!("pty://output/{id}"),
                String::from_utf8_lossy(&bytes).to_string(),
            );
        },
        move |id, code| {
            let _ = exit_app.emit(&format!("pty://exit/{id}"), PtyExitPayload { code });
        },
    )
}

#[tauri::command]
pub fn pty_write(state: State<AppState>, session_id: u32, data: String) -> Result<(), String> {
    state.pty_manager.write(session_id, &data)
}

#[tauri::command]
pub fn pty_resize(
    state: State<AppState>,
    session_id: u32,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.pty_manager.resize(session_id, cols, rows)
}

#[tauri::command]
pub fn pty_kill(state: State<AppState>, session_id: u32) -> Result<(), String> {
    state.pty_manager.kill(session_id)
}
```

- [ ] **Step 3: main.rs 注册命令**

`src-tauri/src/main.rs` 的 `generate_handler![` 列表中 `terminal::resolve_directory,` 之后追加:

```rust
            common::pty::pty_create,
            common::pty::pty_write,
            common::pty::pty_resize,
            common::pty::pty_kill,
```

(`use common::pty;` 不需要——用全路径即可,避免与现有 import 风格冲突。)

- [ ] **Step 4: 验证编译 + 测试仍过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml pty`
Expected: 2 passed,无编译错误。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/common/pty.rs src-tauri/src/app_state.rs src-tauri/src/main.rs
git commit -m "feat: expose PTY sessions as tauri commands with event streaming"
```

---

### Task 4: 前端真终端(xterm.js)

**Files:**
- Create: `src/api/modules/pty.js`
- Create: `src/services/terminal/sessionRuntime.js`
- Rewrite: `src/stores/terminal.js`
- Rewrite: `src/components/ToolWindow/TerminalPanel.vue`
- Modify: `src/api/index.js`
- Modify: `src/components/ToolWindow/BottomPanelHost.vue:27-31`
- Modify: `src/components/Common/MenuBar.vue:227-235` + menus 数组中的终端菜单项
- Modify: `src/App.vue:51`

核心设计:xterm `Terminal` 实例与其 DOM 容器存放在**模块级 Map**(`sessionRuntime.js`),不随组件销毁——`BottomPanelHost` 用 `v-if` 切面板时只是重新挂载容器,PTY 会话与回显缓冲不丢。Pinia store 只存会话元数据(响应式 UI 用)。

- [ ] **Step 1: API 桥 `src/api/modules/pty.js`**

```js
// src/api/modules/pty.js
import { invoke } from '@tauri-apps/api/core'

export function ptyCreate({ shell = null, cwd = null, cols, rows }) {
  return invoke('pty_create', { shell, cwd, cols, rows })
}

export function ptyWrite(sessionId, data) {
  return invoke('pty_write', { sessionId, data })
}

export function ptyResize(sessionId, cols, rows) {
  return invoke('pty_resize', { sessionId, cols, rows })
}

export function ptyKill(sessionId) {
  return invoke('pty_kill', { sessionId })
}
```

`src/api/index.js` 追加一行:`export * from './modules/pty'`

- [ ] **Step 2: 模块级运行时 `src/services/terminal/sessionRuntime.js`**

```js
// src/services/terminal/sessionRuntime.js
// 模块级终端运行时:xterm 实例与 DOM 容器脱离组件生命周期存活。
// 面板被 v-if 卸载/重建时只是重新 append 容器,PTY 会话与回显不中断。
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { listen } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'
import { ptyCreate, ptyWrite, ptyResize, ptyKill } from '../../api'

const runtimes = new Map()

export async function openTerminalSession({ shell = null, cwd = null } = {}) {
  const sessionId = await ptyCreate({ shell, cwd, cols: 80, rows: 24 })

  const term = new Terminal({
    scrollback: 5000,
    fontSize: 13,
    fontFamily: 'Consolas, "Courier New", monospace',
    cursorBlink: true,
    theme: { background: '#111111' }
  })
  const fit = new FitAddon()
  term.loadAddon(fit)

  const container = document.createElement('div')
  container.className = 'absolute inset-0'
  container.style.display = 'none'
  term.open(container)

  term.onData((data) => {
    ptyWrite(sessionId, data)
  })

  const unlistenOutput = await listen(`pty://output/${sessionId}`, ({ payload }) => {
    term.write(payload)
  })
  const unlistenExit = await listen(`pty://exit/${sessionId}`, ({ payload }) => {
    const code = payload?.code
    term.write(`\r\n\x1b[90m[进程已退出${code != null ? `,code ${code}` : ''}]\x1b[0m\r\n`)
  })

  runtimes.set(sessionId, { term, fit, container, unlistenOutput, unlistenExit })
  return sessionId
}

export function mountAllSessions(hostEl) {
  for (const runtime of runtimes.values()) {
    hostEl.appendChild(runtime.container)
  }
}

export function showSession(sessionId) {
  for (const [id, runtime] of runtimes) {
    runtime.container.style.display = id === sessionId ? 'block' : 'none'
  }
  const runtime = runtimes.get(sessionId)
  if (!runtime) return
  // display 切换后等一帧再 fit,否则容器尺寸还是 0
  requestAnimationFrame(() => {
    runtime.fit.fit()
    ptyResize(sessionId, runtime.term.cols, runtime.term.rows)
    runtime.term.focus()
  })
}

export function fitSession(sessionId) {
  const runtime = runtimes.get(sessionId)
  if (!runtime || runtime.container.style.display === 'none') return
  runtime.fit.fit()
  ptyResize(sessionId, runtime.term.cols, runtime.term.rows)
}

export async function disposeTerminalSession(sessionId) {
  const runtime = runtimes.get(sessionId)
  if (!runtime) return
  runtimes.delete(sessionId)
  runtime.unlistenOutput()
  runtime.unlistenExit()
  runtime.term.dispose()
  runtime.container.remove()
  try {
    await ptyKill(sessionId)
  } catch {
    // 进程可能已自然退出,忽略
  }
}
```

- [ ] **Step 3: 重写 `src/stores/terminal.js`**

整文件替换为:

```js
import { defineStore } from 'pinia'
import {
  openTerminalSession,
  showSession,
  disposeTerminalSession
} from '../services/terminal/sessionRuntime'
import { useProjectStore } from './project'

let titleCounter = 0

export const useTerminalStore = defineStore('terminal', {
  state: () => ({
    sessions: [],          // { id, title } — xterm 实例在 sessionRuntime 模块级 Map 里
    activeSessionId: null
  }),

  getters: {
    sessionCount: (state) => state.sessions.length,
    activeSession: (state) =>
      state.sessions.find((session) => session.id === state.activeSessionId) || null
  },

  actions: {
    async openSession() {
      const projectStore = useProjectStore()
      const id = await openTerminalSession({ cwd: projectStore.rootPath || null })
      titleCounter += 1
      this.sessions.push({ id, title: `终端 ${titleCounter}` })
      this.setActive(id)
      return id
    },

    setActive(id) {
      if (!this.sessions.some((session) => session.id === id)) return
      this.activeSessionId = id
      showSession(id)
    },

    async closeSession(id) {
      const index = this.sessions.findIndex((session) => session.id === id)
      if (index === -1) return
      this.sessions.splice(index, 1)
      await disposeTerminalSession(id)
      if (this.activeSessionId === id) {
        const next = this.sessions[index] || this.sessions[index - 1]
        if (next) {
          this.setActive(next.id)
        } else {
          this.activeSessionId = null
        }
      }
    },

    // ---- 会话快照兼容层(useWorkbenchSession 仍会调用) ----
    // PTY 会话本质上不可跨刷新恢复,保留空实现。
    hydrate() {},
    toSnapshot() {
      return {}
    },
    setWorkingDirectory() {}
  }
})
```

注意:`useProjectStore` 须确认 `rootPath` 字段名——执行时打开 `src/stores/project.js` 核对(App.vue:45 已出现 `projectStore.rootPath`,应当一致)。

- [ ] **Step 4: 重写 `src/components/ToolWindow/TerminalPanel.vue`**

整文件替换为:

```vue
<template>
  <div class="h-full flex flex-col bg-[#181818] border-t border-black">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="flex items-center gap-1 min-w-0 overflow-x-auto">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-400 mr-2">Terminal</span>
        <button
          v-for="session in terminalStore.sessions"
          :key="session.id"
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border flex items-center gap-1 shrink-0"
          :class="session.id === terminalStore.activeSessionId
            ? 'text-gray-100 border-[#3b82f6] bg-transparent'
            : 'text-gray-400 border-transparent hover:text-gray-200 hover:border-[#3b82f6]/55'"
          @click="terminalStore.setActive(session.id)"
        >
          {{ session.title }}
          <span
            class="text-gray-500 hover:text-red-400"
            @click.stop="terminalStore.closeSession(session.id)"
          >×</span>
        </button>
        <button
          type="button"
          class="h-6 px-2 text-[12px] rounded-sm text-gray-400 hover:text-gray-200 hover:bg-white/8 shrink-0"
          title="新建终端"
          @click="terminalStore.openSession()"
        >
          ＋
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
          @click="workbenchPanelsStore.hideBottomPanel()"
        >
          隐藏
        </button>
      </div>
    </div>

    <div ref="hostRef" class="flex-1 min-h-0 relative bg-[#111111]">
      <div
        v-if="!terminalStore.sessionCount"
        class="h-full flex items-center justify-center text-sm text-gray-500"
      >
        没有打开的终端 — 点击 ＋ 新建
      </div>
    </div>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { mountAllSessions, showSession, fitSession } from '../../services/terminal/sessionRuntime'

const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const hostRef = ref(null)
let resizeObserver = null

onMounted(() => {
  // 重新挂载模块级容器(面板可能被 v-if 重建)
  mountAllSessions(hostRef.value)
  if (terminalStore.activeSessionId != null) {
    showSession(terminalStore.activeSessionId)
  } else if (!terminalStore.sessionCount) {
    terminalStore.openSession()
  }

  resizeObserver = new ResizeObserver(() => {
    if (terminalStore.activeSessionId != null) {
      fitSession(terminalStore.activeSessionId)
    }
  })
  resizeObserver.observe(hostRef.value)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})
</script>
```

- [ ] **Step 5: 修补消费方**

1. `src/App.vue:51` — `<span>{{ terminalStore.shell.toUpperCase() }}</span>` 替换为:

```html
                <span v-if="terminalStore.sessionCount">{{ terminalStore.sessionCount }} 个终端</span>
```

2. `src/components/Common/MenuBar.vue:227-235` — 三个 case(`terminal.usePwsh` / `terminal.useCmd` / `terminal.clear`)替换为:

```js
    case 'terminal.new':
      workbenchPanelsStore.showBottomPanel('terminal')
      terminalStore.openSession()
      break
```

并在 `menus` 计算属性中找到引用 `terminal.usePwsh`/`terminal.useCmd`/`terminal.clear` 的菜单项(用 grep 定位),替换为单项:

```js
        { label: '新建终端', key: 'terminal.new' }
```

3. `src/components/ToolWindow/BottomPanelHost.vue:27-31` — 终端面板改 `v-show` 常驻(双保险;主要存活机制是模块级运行时):

```html
    <div class="flex-1 min-h-0">
      <div v-show="workbenchPanelsStore.activeBottomPanel === 'terminal'" class="h-full">
        <TerminalPanel />
      </div>
      <OutputPanel v-if="workbenchPanelsStore.activeBottomPanel === 'output'" />
      <ProblemsPanel v-else-if="workbenchPanelsStore.activeBottomPanel === 'problems'" />

      <div
        v-else-if="workbenchPanelsStore.activeBottomPanel !== 'terminal'"
        class="h-full flex items-center justify-center text-sm text-gray-500 bg-[#111111]"
      >
        {{ currentPanelLabel }} 面板尚未实现
      </div>
    </div>
```

- [ ] **Step 6: 构建验证**

Run: `npm run build`
Expected: 构建成功,无 import 报错。grep 检查无残留旧 API 调用:
`grep -rn "runShellCommand\|terminalStore.execute\|terminalStore.shell\|clearOutput()" src/ --include="*.vue" --include="*.js"` — `terminalStore.` 开头的引用应为 0 条(`runShellCommand` 在 `api/modules/terminal.js` 的定义保留)。

- [ ] **Step 7: Commit**

```bash
git add src/api/modules/pty.js src/api/index.js src/services/terminal/sessionRuntime.js \
  src/stores/terminal.js src/components/ToolWindow/TerminalPanel.vue \
  src/components/ToolWindow/BottomPanelHost.vue src/components/Common/MenuBar.vue src/App.vue
git commit -m "feat: real multi-session xterm.js terminal backed by PTY"
```

---

### Task 5: map_parser 解析全局寄存器(TDD)

**Files:**
- Modify: `src-tauri/src/modules/ecl/map_parser.rs`

eclmap 还有 `!gvar_names`(`ID NAME`,ID 可为负)与 `!gvar_types`(`ID $`=int / `ID %`=float)段,AI 辅助包需要。同时把解析逻辑从文件 IO 拆出便于测试。

- [ ] **Step 1: 写失败测试**

在 `map_parser.rs` 末尾追加:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"!eclmap
!ins_names
10 jump
11 callSub
!ins_signatures
10 ot
11 m
!gvar_names
-9985 I0
-9982 F0
!gvar_types
-9985 $
-9982 %
"#;

    #[test]
    fn parses_instructions_and_globals() {
        let data = parse_ecl_map_content("maps/th17.eclm", SAMPLE).expect("parse");
        assert_eq!(data.instructions.len(), 2);
        assert_eq!(data.instructions[0].name, "jump");
        assert_eq!(data.instructions[0].signature.as_deref(), Some("ot"));

        assert_eq!(data.globals.len(), 2);
        let i0 = &data.globals[0];
        assert_eq!(i0.id, -9985);
        assert_eq!(i0.name, "I0");
        assert_eq!(i0.var_type, "int");
        let f0 = &data.globals[1];
        assert_eq!(f0.id, -9982);
        assert_eq!(f0.var_type, "float");
    }

    #[test]
    fn gvar_without_type_is_unknown() {
        let sample = "!gvar_names\n-1 X\n";
        let data = parse_ecl_map_content("maps/th17.eclm", sample).expect("parse");
        assert_eq!(data.globals[0].var_type, "unknown");
    }
}
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml map_parser`
Expected: 编译错误(`parse_ecl_map_content` / `globals` 不存在)。

- [ ] **Step 3: 实现**

1. 新类型(`EclMapSemanticData` 定义前):

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclMapGlobalVar {
    pub id: i32,
    pub name: String,
    pub var_type: String, // "int" | "float" | "unknown"
}
```

2. `EclMapSemanticData` 增加字段 `pub globals: Vec<EclMapGlobalVar>,`(追加在 `builtins` 后;serde 为加法变更,前端不受影响)。

3. gvar 行正则(负数 ID):

```rust
fn gvar_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^(-?\d+)\s+(\S+)").expect("valid regex")
    })
}
```

4. 把 `parse_ecl_map_file` 拆为两个函数:

```rust
pub fn parse_ecl_map_file(path: &str) -> Result<EclMapSemanticData, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("Failed to read eclmap file: {error}"))?;
    parse_ecl_map_content(path, &content)
}

pub fn parse_ecl_map_content(path: &str, content: &str) -> Result<EclMapSemanticData, String> {
    // 原 parse_ecl_map_file 函数体移到这里,并做以下扩展:
}
```

5. 在解析循环中增加 gvar 段状态(与 `in_instruction_names` 平行):

```rust
    let mut in_gvar_names = false;
    let mut in_gvar_types = false;
    let mut globals: Vec<EclMapGlobalVar> = Vec::new();
    let mut gvar_types: BTreeMap<i32, String> = BTreeMap::new();
```

段切换处(`!ins_signatures` 分支后)插入:

```rust
        if line.starts_with("!gvar_names") {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = true;
            in_gvar_types = false;
            current_section = None;
            continue;
        }

        if line.starts_with("!gvar_types") {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = false;
            in_gvar_types = true;
            current_section = None;
            continue;
        }
```

**重要**:原有的通配段重置分支(`line.starts_with("!") && !line.starts_with("!ins_names") && ...`)需同步排除 gvar 段,改为:

```rust
        if line.starts_with('!')
            && !line.starts_with("!ins_names")
            && !line.starts_with("!ins_signatures")
            && !line.starts_with("!gvar_names")
            && !line.starts_with("!gvar_types")
        {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = false;
            in_gvar_types = false;
            current_section = None;
            continue;
        }
```

行处理(`in_instruction_signatures` 块后)追加:

```rust
        if in_gvar_names {
            if let Some(captures) = gvar_line_regex().captures(line) {
                let id = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<i32>().ok())
                    .unwrap_or_default();
                let name = captures.get(2).map(|v| v.as_str()).unwrap_or_default();
                globals.push(EclMapGlobalVar {
                    id,
                    name: name.to_string(),
                    var_type: "unknown".to_string(),
                });
            }
            continue;
        }

        if in_gvar_types {
            if let Some(captures) = gvar_line_regex().captures(line) {
                let id = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<i32>().ok())
                    .unwrap_or_default();
                let type_mark = captures.get(2).map(|v| v.as_str()).unwrap_or_default();
                let var_type = match type_mark {
                    "$" => "int",
                    "%" => "float",
                    _ => "unknown",
                };
                gvar_types.insert(id, var_type.to_string());
            }
            continue;
        }
```

收尾(instructions 签名回填循环后)补:

```rust
    for global in &mut globals {
        if let Some(var_type) = gvar_types.get(&global.id) {
            global.var_type = var_type.clone();
        }
    }
```

返回值补 `globals,` 字段。

- [ ] **Step 4: 运行确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml map_parser`
Expected: 2 passed。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/modules/ecl/map_parser.rs
git commit -m "feat: parse eclmap global registers (!gvar_names/!gvar_types)"
```

---

### Task 6: .mcp.json 非破坏性合并(TDD)

**Files:**
- Create: `src-tauri/src/common/mcp_config.rs`
- Modify: `src-tauri/src/common/mod.rs`(追加 `pub mod mcp_config;`)

- [ ] **Step 1: 写失败测试**

创建 `src-tauri/src/common/mcp_config.rs`:

```rust
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_mcp_json_when_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        upsert_mcp_entry(&root, 12345, "tok-abc").expect("upsert");

        let content = fs::read_to_string(dir.path().join(".mcp.json")).expect("read");
        let value: Value = serde_json::from_str(&content).expect("json");
        assert_eq!(
            value["mcpServers"]["thtk-studio"]["url"],
            "http://127.0.0.1:12345/mcp"
        );
        assert_eq!(
            value["mcpServers"]["thtk-studio"]["headers"]["Authorization"],
            "Bearer tok-abc"
        );
        assert_eq!(value["mcpServers"]["thtk-studio"]["type"], "http");
    }

    #[test]
    fn preserves_existing_servers_and_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"my-tool":{"type":"stdio","command":"foo"}},"custom":1}"#,
        )
        .expect("seed");

        upsert_mcp_entry(&root, 999, "t").expect("upsert");

        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["mcpServers"]["my-tool"]["command"], "foo");
        assert_eq!(value["custom"], 1);
        assert_eq!(value["mcpServers"]["thtk-studio"]["type"], "http");
    }

    #[test]
    fn refuses_to_clobber_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(dir.path().join(".mcp.json"), "{not json").expect("seed");

        let result = upsert_mcp_entry(&root, 1, "t");

        assert!(result.is_err());
        // 原文件保持原样
        assert_eq!(
            fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            "{not json"
        );
    }
}
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mcp_config`
Expected: 编译错误(`upsert_mcp_entry` 未定义)。

- [ ] **Step 3: 实现(tests 模块上方)**

```rust
/// 在项目根的 .mcp.json 中写入/更新 thtk-studio 这一个 server entry,
/// 不动用户已有的其他 server 和顶层键。文件不是合法 JSON 时报错而非覆盖。
pub fn upsert_mcp_entry(project_root: &str, port: u16, token: &str) -> Result<(), String> {
    let path = Path::new(project_root).join(".mcp.json");

    let mut root: Value = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read .mcp.json: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!(".mcp.json is not valid JSON, refusing to overwrite: {e}"))?
    } else {
        json!({})
    };

    let root_object = root
        .as_object_mut()
        .ok_or_else(|| ".mcp.json top level is not an object, refusing to overwrite".to_string())?;

    let servers = root_object
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    let servers_object = servers
        .as_object_mut()
        .ok_or_else(|| ".mcp.json mcpServers is not an object".to_string())?;

    servers_object.insert(
        "thtk-studio".to_string(),
        json!({
            "type": "http",
            "url": format!("http://127.0.0.1:{port}/mcp"),
            "headers": { "Authorization": format!("Bearer {token}") }
        }),
    );

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize .mcp.json: {e}"))?;
    fs::write(&path, serialized).map_err(|e| format!("Failed to write .mcp.json: {e}"))
}
```

`src-tauri/src/common/mod.rs` 追加 `pub mod mcp_config;`。

- [ ] **Step 4: 运行确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mcp_config`
Expected: 3 passed。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/common/mcp_config.rs src-tauri/src/common/mod.rs
git commit -m "feat: non-destructive .mcp.json upsert for thtk-studio entry"
```

---

### Task 7: MCP 工具核心逻辑(可测自由函数)

**Files:**
- Create: `src-tauri/src/modules/mcp/mod.rs`
- Create: `src-tauri/src/modules/mcp/tools.rs`
- Modify: `src-tauri/src/modules/mod.rs`(追加 `pub mod mcp;`)
- Modify: `src-tauri/src/modules/ecl/commands.rs:16`(`resolve_default_maps` 改 `pub(crate)`)

工具逻辑写成接收 `&AppConfig` 等普通参数的自由函数,server 层只做参数解包——单测不需要 Tauri/HTTP。

- [ ] **Step 1: 模块骨架**

`src-tauri/src/modules/mod.rs` 追加 `pub mod mcp;`。
创建 `src-tauri/src/modules/mcp/mod.rs`:

```rust
pub mod server;
pub mod tools;
```

(`server.rs` 在 Task 8 创建,本 task 先建空文件 `src-tauri/src/modules/mcp/server.rs` 内容为空注释 `// MCP server bootstrap — implemented in Task 8`,保证编译。)

`src-tauri/src/modules/ecl/commands.rs:16` — `fn resolve_default_maps(` 改为 `pub(crate) fn resolve_default_maps(`。

- [ ] **Step 2: 写失败测试**

创建 `src-tauri/src/modules/mcp/tools.rs`,文件骨架 + 测试:

```rust
use crate::common::toolchain;
use crate::config::AppConfig;
use crate::modules::ecl::{commands, compiler, map_parser};
use serde_json::{json, Value};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SAMPLE_MAP: &str = "!eclmap\n!ins_names\n10 jump\n23 wait\n!ins_signatures\n10 ot\n23 S\n!gvar_names\n-9985 I0\n!gvar_types\n-9985 $\n";

    fn write_sample_map(dir: &tempfile::TempDir) -> String {
        let path = dir.path().join("th17.eclm");
        let mut file = std::fs::File::create(&path).expect("create map");
        file.write_all(SAMPLE_MAP.as_bytes()).expect("write map");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn lookup_by_name_substring() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);

        let result = lookup_semantics(&map_path, "wai").expect("lookup");

        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0]["name"], "wait");
        assert_eq!(matches[0]["opcode"], 23);
    }

    #[test]
    fn lookup_by_opcode() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);

        let result = lookup_semantics(&map_path, "10").expect("lookup");

        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches[0]["name"], "jump");
        // 全局寄存器也参与查询
        let result = lookup_semantics(&map_path, "I0").expect("lookup");
        assert_eq!(result["globals"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn workspace_info_reports_root_and_tools() {
        let config = AppConfig::default();
        let info = workspace_info(&config, Some("/tmp/proj"));
        assert_eq!(info["projectRoot"], "/tmp/proj");
        assert_eq!(info["toolchains"].as_array().expect("array").len(), 5);
    }
}
```

注:`AppConfig::default()` 若未实现 `Default`,执行时打开 `src-tauri/src/config.rs` 确认——若没有,给 `AppConfig` derive 加 `Default`(字段全为 String/map,可安全派生;`#[serde(default)]` 已普遍存在的 MVP 配置结构通常可加)。

- [ ] **Step 3: 运行确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mcp::tools`
Expected: 编译错误(函数未定义)。

- [ ] **Step 4: 实现(tests 上方)**

```rust
/// get_workspace_info:项目根 + 五个工具链状态
pub fn workspace_info(config: &AppConfig, project_root: Option<&str>) -> Value {
    let toolchains: Vec<Value> = toolchain::get_all_toolchain_statuses(config)
        .into_iter()
        .map(|status| {
            json!({
                "tool": status.tool,
                "available": status.available,
                "version": status.version,
                "resolvedPath": status.resolved_path,
                "message": status.message,
            })
        })
        .collect();

    json!({
        "projectRoot": project_root,
        "defaultGameVersion": config.default_game_version,
        "toolchains": toolchains,
    })
}

fn run_thecl(config: &AppConfig, request: &compiler::TheclRequest) -> Value {
    let result = compiler::run(config, request);
    json!({
        "success": result.success,
        "mode": result.mode,
        "inputPath": result.input_path,
        "outputPath": result.output_path,
        "diagnostics": result.diagnostics,
        "message": result.message,
    })
}

fn resolved_maps(config: &AppConfig, version: &str) -> Vec<String> {
    commands::resolve_default_maps(config, &config.thtk_dir.clone(), version, Vec::new())
}

/// check_ecl:编译到临时文件并删除产物,只为拿诊断
pub fn check_ecl(config: &AppConfig, source_path: &str) -> Result<Value, String> {
    let version = config.default_game_version.clone();
    let file_name = Path::new(source_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("Invalid source path: {source_path}"))?;
    let temp_output = std::env::temp_dir().join(format!("thtk-check-{file_name}.ecl"));

    let request = compiler::TheclRequest {
        mode: compiler::TheclMode::Compile,
        version: version.clone(),
        input_path: source_path.to_string(),
        output_path: Some(temp_output.to_string_lossy().to_string()),
        map_paths: resolved_maps(config, &version),
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    };

    let mut value = run_thecl(config, &request);
    let _ = std::fs::remove_file(&temp_output);
    // 检查模式不暴露产物路径
    if let Some(object) = value.as_object_mut() {
        object.insert("outputPath".to_string(), Value::Null);
        object.insert("checkOnly".to_string(), json!(true));
    }
    Ok(value)
}

/// compile_ecl:真编译,产物落盘
pub fn compile_ecl(
    config: &AppConfig,
    source_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    let version = config.default_game_version.clone();
    let request = compiler::TheclRequest {
        mode: compiler::TheclMode::Compile,
        version: version.clone(),
        input_path: source_path.to_string(),
        output_path,
        map_paths: resolved_maps(config, &version),
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    };
    Ok(run_thecl(config, &request))
}

/// decompile_ecl
pub fn decompile_ecl(
    config: &AppConfig,
    binary_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    let version = config.default_game_version.clone();
    let request = compiler::TheclRequest {
        mode: compiler::TheclMode::Decompile,
        version: version.clone(),
        input_path: binary_path.to_string(),
        output_path,
        map_paths: resolved_maps(config, &version),
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    };
    Ok(run_thecl(config, &request))
}

/// lookup_ecl_semantics:按名称子串或精确 opcode 查指令,同时匹配全局寄存器
pub fn lookup_semantics(map_path: &str, query: &str) -> Result<Value, String> {
    let data = map_parser::parse_ecl_map_file(map_path)?;
    let query_lower = query.trim().to_lowercase();
    let opcode_query = query_lower.parse::<i64>().ok();

    let instructions: Vec<Value> = data
        .instructions
        .iter()
        .filter(|ins| {
            ins.name.to_lowercase().contains(&query_lower)
                || opcode_query == Some(ins.opcode as i64)
        })
        .take(50)
        .map(|ins| {
            json!({
                "opcode": ins.opcode,
                "name": ins.name,
                "section": ins.section,
                "signature": ins.signature,
                "params": ins.params,
            })
        })
        .collect();

    let globals: Vec<Value> = data
        .globals
        .iter()
        .filter(|g| {
            g.name.to_lowercase().contains(&query_lower) || opcode_query == Some(g.id as i64)
        })
        .take(50)
        .map(|g| json!({ "id": g.id, "name": g.name, "type": g.var_type }))
        .collect();

    Ok(json!({
        "mapPath": map_path,
        "version": data.version,
        "query": query,
        "instructions": instructions,
        "globals": globals,
    }))
}

/// 解析当前应使用的 eclmap 路径(配置优先,其次 thtk_dir/maps/{ver}.eclmap)
pub fn resolve_map_path(config: &AppConfig) -> Result<String, String> {
    let maps = resolved_maps(config, &config.default_game_version);
    maps.into_iter().next().ok_or_else(|| {
        "No eclmap configured: set eclmap_path or thtk_dir in settings".to_string()
    })
}
```

- [ ] **Step 5: 运行确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml mcp::tools`
Expected: 3 passed。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/modules/mcp/ src-tauri/src/modules/mod.rs src-tauri/src/modules/ecl/commands.rs
git commit -m "feat: MCP tool core logic as testable free functions"
```

---

### Task 8: MCP server(rmcp)+ 启动接线 + .mcp.json 写入

**Files:**
- Rewrite: `src-tauri/src/modules/mcp/server.rs`
- Modify: `src-tauri/src/app_state.rs`(增加 `mcp_server` 字段)
- Modify: `src-tauri/src/main.rs`(setup 启动 server;`set_project_root` 写 .mcp.json)

rmcp 1.7 API 参考官方示例 `counter_streamhttp.rs`(StreamableHttpService + LocalSessionManager + axum nest_service)。若编译时 API 名有出入,以 `https://docs.rs/rmcp/1.7.0` 为准修正,不改变结构。

- [ ] **Step 1: 实现 server.rs**

```rust
use crate::app_state::AppState;
use crate::modules::mcp::tools;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData as McpError,
};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};

pub struct McpServerInfo {
    pub port: u16,
    pub token: String,
}

#[derive(Clone)]
pub struct ThtkMcp {
    app: AppHandle,
    tool_router: ToolRouter<Self>,
}

fn json_result(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// spec §6:工具错误统一带结构化 data {code, message, hint}
fn tool_error(message: String) -> McpError {
    let hint = if message.contains("not configured") {
        "在 THTK-Studio 设置中配置 thtk 工具路径 (thtk_dir / thecl_path / eclmap_path)"
    } else {
        "检查文件路径是否为绝对路径,以及 get_workspace_info 中工具链是否可用"
    };
    McpError::internal_error(
        message.clone(),
        Some(serde_json::json!({
            "code": "tool_failed",
            "message": message,
            "hint": hint,
        })),
    )
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SourcePathParams {
    /// .decl 源文件的绝对路径
    pub source_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompileParams {
    /// .decl 源文件的绝对路径
    pub source_path: String,
    /// 可选的输出 .ecl 路径;省略时按约定推断(.decl → .ecl)
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DecompileParams {
    /// .ecl 二进制文件的绝对路径
    pub binary_path: String,
    /// 可选的输出 .decl 路径
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupParams {
    /// 指令名子串(如 "wait")、精确 opcode(如 "23")或寄存器名(如 "I0")
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReportParams {
    /// 报告标题(显示在输出面板分组上)
    pub title: String,
    /// 报告正文,支持多行
    pub body: String,
    /// "info" | "success" | "warning" | "error"
    pub level: Option<String>,
    /// 关联文件的绝对路径(可选)
    pub path: Option<String>,
}

#[tool_router]
impl ThtkMcp {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
        }
    }

    fn config(&self) -> crate::config::AppConfig {
        self.app.state::<AppState>().config_manager.get_config()
    }

    fn project_root(&self) -> Option<String> {
        self.app
            .state::<AppState>()
            .current_project_root
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    #[tool(
        description = "获取 THTK-Studio 工作区信息:项目根目录、默认游戏版本、五个 thtk 工具(thecl/thmsg/thanm/thstd/thdat)的可用状态与版本。"
    )]
    async fn get_workspace_info(&self) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        json_result(tools::workspace_info(&config, root.as_deref()))
    }

    #[tool(
        description = "编译检查一个 ECL 源文件(.decl):运行 thecl 编译但不保留产物,返回结构化诊断列表(与 IDE 问题面板同源)。改完代码后用它验证。"
    )]
    async fn check_ecl(
        &self,
        Parameters(params): Parameters<SourcePathParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        json_result(tools::check_ecl(&config, &params.source_path).map_err(tool_error)?)
    }

    #[tool(
        description = "编译 ECL 源文件(.decl → .ecl),产物落盘,返回诊断与产物路径。"
    )]
    async fn compile_ecl(
        &self,
        Parameters(params): Parameters<CompileParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        json_result(
            tools::compile_ecl(&config, &params.source_path, params.output_path)
                .map_err(tool_error)?,
        )
    }

    #[tool(description = "反编译 ECL 二进制(.ecl → .decl 文本),返回诊断与产物路径。")]
    async fn decompile_ecl(
        &self,
        Parameters(params): Parameters<DecompileParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        json_result(
            tools::decompile_ecl(&config, &params.binary_path, params.output_path)
                .map_err(tool_error)?,
        )
    }

    #[tool(
        description = "按指令名子串、opcode 或寄存器名查询当前 eclmap 语义数据(签名、参数、全局寄存器)。写 ECL 前先查签名。"
    )]
    async fn lookup_ecl_semantics(
        &self,
        Parameters(params): Parameters<LookupParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let map_path = tools::resolve_map_path(&config).map_err(tool_error)?;
        json_result(tools::lookup_semantics(&map_path, &params.query).map_err(tool_error)?)
    }

    #[tool(
        description = "向 IDE 用户的输出面板推送一张结构化报告卡片。完成一项工作或发现重要问题时,用它向人类汇报结论。"
    )]
    async fn report_to_user(
        &self,
        Parameters(params): Parameters<ReportParams>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .emit(
                "mcp://report",
                serde_json::json!({
                    "title": params.title,
                    "body": params.body,
                    "level": params.level.unwrap_or_else(|| "info".to_string()),
                    "path": params.path,
                }),
            )
            .map_err(|e| McpError::internal_error(format!("emit failed: {e}"), None))?;
        json_result(serde_json::json!({ "delivered": true }))
    }
}

#[tool_handler]
impl ServerHandler for ThtkMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "THTK-Studio:东方 Project 脚本魔改 IDE。\
                 工作流:decompile_ecl 把 .ecl 反编译为 .decl 文本 → 编辑 → check_ecl 验证 → compile_ecl 产出。\
                 写 ECL 指令前先 lookup_ecl_semantics 查签名;完成任务后 report_to_user 向用户汇报。"
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// 绑定 127.0.0.1 随机端口,带 bearer token 校验,返回端口与 token。
/// serve 循环在 tauri 异步运行时中后台运行。
pub async fn start(app: AppHandle) -> Result<McpServerInfo, String> {
    use axum::response::IntoResponse;
    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    };

    let token = uuid::Uuid::new_v4().simple().to_string();

    let service_app = app.clone();
    let service = StreamableHttpService::new(
        move || Ok(ThtkMcp::new(service_app.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let expected = format!("Bearer {token}");
    let auth_layer = axum::middleware::from_fn(
        move |req: axum::extract::Request, next: axum::middleware::Next| {
            let expected = expected.clone();
            async move {
                let authorized = req
                    .headers()
                    .get(axum::http::header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    == Some(expected.as_str());
                if authorized {
                    next.run(req).await
                } else {
                    axum::http::StatusCode::UNAUTHORIZED.into_response()
                }
            }
        },
    );

    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(auth_layer);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("MCP server bind failed: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("MCP server addr failed: {e}"))?
        .port();

    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            eprintln!("[mcp] server stopped: {error}");
        }
    });

    Ok(McpServerInfo { port, token })
}
```

- [ ] **Step 2: AppState 增加 mcp_server 字段**

`src-tauri/src/app_state.rs` — `pty_manager` 后追加:

```rust
    // MCP server 运行信息 (端口 + token),启动成功后写入
    pub mcp_server: Mutex<Option<crate::modules::mcp::server::McpServerInfo>>,
```

`new()` 中:

```rust
            mcp_server: Mutex::new(None),
```

注意:`app_state.rs` 引用了 `crate::modules`,而 `mod modules` 声明在 main.rs——Rust 中声明顺序无关,可直接编译。

- [ ] **Step 3: main.rs 启动 server + set_project_root 写 .mcp.json**

1. `main()` 的 `tauri::Builder::default()` 链上、`.manage(app_state)` 之后插入:

```rust
        .setup(|app| {
            // MCP server:绑定很快,同步等待拿到端口;失败不阻止应用启动
            let handle = app.handle().clone();
            match tauri::async_runtime::block_on(modules::mcp::server::start(handle)) {
                Ok(info) => {
                    let state = app.state::<AppState>();
                    *state.mcp_server.lock().unwrap_or_else(|e| e.into_inner()) = Some(info);
                }
                Err(error) => {
                    eprintln!("[mcp] failed to start MCP server, agent tools unavailable: {error}");
                }
            }
            Ok(())
        })
```

需要 `use tauri::Manager;`(`app.state` 来自该 trait;若已有则不重复)。

2. `set_project_root`(main.rs:83-89)在 `file_watcher::start_watching(...)` 之后追加:

```rust
    // 项目根就绪后,把 MCP server 接入信息写进 .mcp.json(非破坏性)
    let mcp = state.mcp_server.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(info) = mcp.as_ref() {
        if let Err(error) = common::mcp_config::upsert_mcp_entry(&path, info.port, &info.token) {
            eprintln!("[mcp] failed to update .mcp.json: {error}");
        }
    }
```

- [ ] **Step 4: 编译 + 全量测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: 全部通过(pty 2 + map_parser 2 + mcp_config 3 + mcp::tools 3)。若 rmcp 宏/路径编译失败,对照 `docs.rs/rmcp/1.7.0` 与官方 `examples/servers/src/common/counter.rs` 修正 import 路径(结构不变)。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/modules/mcp/server.rs src-tauri/src/app_state.rs src-tauri/src/main.rs
git commit -m "feat: in-process rmcp streamable HTTP server with six tools + .mcp.json wiring"
```

---

### Task 9: report_to_user 前端桥

**Files:**
- Create: `src/composables/useMcpBridge.js`
- Modify: `src/App.vue`(import + 调用)

- [ ] **Step 1: 创建 composable**

```js
// src/composables/useMcpBridge.js
// 监听 Rust MCP server 转发的 agent 报告,落入输出/问题面板。
import { onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import { useWorkbenchPanelsStore } from '../stores/workbenchPanels'

export function useMcpBridge() {
  const reportsStore = useWorkbenchReportsStore()
  const workbenchPanelsStore = useWorkbenchPanelsStore()
  let unlisten = null

  listen('mcp://report', ({ payload }) => {
    if (!payload) return
    reportsStore.publishToolResult({
      ownerKey: `agent:${payload.title || 'report'}:${Date.now()}`,
      source: 'agent',
      operation: 'report',
      scriptKind: 'text',
      title: payload.title || 'Agent 报告',
      path: payload.path || null,
      success: payload.level !== 'error',
      message: payload.body || '',
      diagnostics: []
    })
    // agent 主动汇报时把输出面板带到前台,让人类看到
    workbenchPanelsStore.showBottomPanel('output')
  }).then((fn) => {
    unlisten = fn
  })

  onBeforeUnmount(() => {
    unlisten?.()
  })
}
```

- [ ] **Step 2: App.vue 接线**

`src/App.vue` import 区(`useFileWatcher` 行后)加:

```js
import { useMcpBridge } from './composables/useMcpBridge'
```

`useFileWatcher({...})` 调用之后加:

```js
useMcpBridge()
```

- [ ] **Step 3: 构建验证**

Run: `npm run build`
Expected: 成功。

- [ ] **Step 4: Commit**

```bash
git add src/composables/useMcpBridge.js src/App.vue
git commit -m "feat: surface agent report_to_user cards in output panel"
```

---

### Task 10: AI 辅助包生成(skill 脚手架 + references 导出)

**Files:**
- Create: `src-tauri/assets/ecl-skill-template.md`
- Create: `src-tauri/src/modules/ecl/ai_pack.rs`
- Modify: `src-tauri/src/modules/ecl/mod.rs`(追加 `pub mod ai_pack;`)
- Modify: `src-tauri/src/modules/ecl/commands.rs`(新命令)
- Modify: `src-tauri/src/main.rs`(注册 `generate_ai_assist_pack`)
- Modify: `src/api/modules/eclMap.js`(新 API)
- Modify: `src/components/Common/MenuBar.vue`(菜单项)

- [ ] **Step 1: skill 模板资产**

创建 `src-tauri/assets/ecl-skill-template.md`(`{{VERSION}}`/`{{INSTRUCTIONS_FILE}}`/`{{REGISTERS_FILE}}` 为生成时替换的占位符):

```markdown
---
name: ecl-modding
description: 在本项目中编写或修改东方 Project ECL 脚本(.decl)时使用 — ECL 语言核心概念、thtk 工作流、何时查 references 与何时调用 thtk-studio MCP 工具
---

# ECL 脚本魔改指南(th{{VERSION}})

## ECL 是什么

ECL 控制东方 Project 原作的敌机行为与弹幕逻辑。二进制 `.ecl` 文件由 thecl
反编译为文本(本项目约定扩展名 `.decl`),修改后再编译回 `.ecl`。

## 核心概念

- **sub(子程序)**:`void sub_name(...)` 形式的过程,敌机/弹幕逻辑的基本单元。
  关卡主流程通常从 `main` 系列 sub 开始,boss 行为在专门的 sub 里。
- **时间标签**:行首的 `数字:` 表示"等到本 sub 时钟达到该帧再继续"。
  60 = 1 秒(60fps)。这是 ECL 时序的核心——指令按时间标签调度,不是顺序立即执行。
- **难度分支**:`!EN`、`!HL`、`!*` 等行首标记限定后续行只在指定难度生效
  (E=Easy,N=Normal,H=Hard,L=Lunatic,*=全部)。
- **变量与寄存器**:
  - 局部变量:`int x = 0;` / `float y = 1.0;`
  - 全局寄存器:`I0`-`I3`(int)、`F0`-`F3`(float)等,以及大量只读系统寄存器
    (玩家坐标、随机数等)。完整列表见 references/{{REGISTERS_FILE}}。
  - `$` 前缀按 int 读,`%` 前缀按 float 读。
- **指令(ins)**:`ins_23(60)` 或经 eclmap 映射后的别名 `wait(60)`。
  同一 opcode 在不同作品中含义可能不同——**不要凭记忆写指令,先查签名**。

## 工作流(重要)

1. 反编译:用 MCP 工具 `decompile_ecl` 把 .ecl 转成 .decl 文本
2. 编辑 .decl(UTF-8;原始游戏文本另行处理,保持既有编码不动)
3. **每次修改后**用 `check_ecl` 验证——返回的诊断与 IDE 问题面板同源
4. 通过后用 `compile_ecl` 产出 .ecl
5. 完成一项任务后用 `report_to_user` 向用户汇报做了什么、验证结果如何

## 何时查什么

| 需求 | 途径 |
|---|---|
| 浏览/检索本作全部指令 | 读 references/{{INSTRUCTIONS_FILE}}(可 grep) |
| 单条指令的精确签名 | MCP 工具 `lookup_ecl_semantics`(name 子串或 opcode) |
| 全局寄存器含义 | references/{{REGISTERS_FILE}} |
| 工具链是否可用 | MCP 工具 `get_workspace_info` |

## 注意事项

- 本文件由 THTK-Studio 生成,references/ 会随 eclmap 重新生成而刷新;
  SKILL.md 可手工补充项目特有约定(不会被覆盖)。
- 不要手工调用 thecl 命令行——MCP 工具封装了版本号、eclmap 与编码参数。
- 时间标签与难度标记是行级前缀,移动代码时务必一并移动。
```

- [ ] **Step 2: 写失败测试**

创建 `src-tauri/src/modules/ecl/ai_pack.rs`:

```rust
use super::map_parser::EclMapSemanticData;
use serde::Serialize;
use std::fs;
use std::path::Path;

const SKILL_TEMPLATE: &str = include_str!("../../../assets/ecl-skill-template.md");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPackResult {
    pub skill_path: String,
    pub skill_written: bool,
    pub reference_files: Vec<String>,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ecl::map_parser;

    const SAMPLE_MAP: &str = "!eclmap\n!ins_names\n10 jump\n23 wait\n!ins_signatures\n10 ot\n23 S\n!gvar_names\n-9985 I0\n!gvar_types\n-9985 $\n";

    fn sample_semantics() -> map_parser::EclMapSemanticData {
        map_parser::parse_ecl_map_content("maps/th17.eclm", SAMPLE_MAP).expect("parse")
    }

    #[test]
    fn generates_skill_and_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        let result = generate(&root, &sample_semantics()).expect("generate");

        assert!(result.skill_written);
        let skill = fs::read_to_string(dir.path().join(".claude/skills/ecl-modding/SKILL.md"))
            .expect("skill exists");
        assert!(skill.contains("th17"), "version substituted");
        assert!(!skill.contains("{{VERSION}}"), "no placeholder left");

        let instructions = fs::read_to_string(
            dir.path()
                .join(".claude/skills/ecl-modding/references/th17-instructions.md"),
        )
        .expect("instructions exist");
        assert!(instructions.contains("wait"));
        assert!(instructions.contains("| 23 |"));

        let registers = fs::read_to_string(
            dir.path()
                .join(".claude/skills/ecl-modding/references/th17-registers.md"),
        )
        .expect("registers exist");
        assert!(registers.contains("I0"));
    }

    #[test]
    fn rerun_preserves_user_skill_but_refreshes_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        generate(&root, &sample_semantics()).expect("first run");

        let skill_path = dir.path().join(".claude/skills/ecl-modding/SKILL.md");
        fs::write(&skill_path, "USER EDITED").expect("user edit");
        let ref_path = dir
            .path()
            .join(".claude/skills/ecl-modding/references/th17-instructions.md");
        fs::write(&ref_path, "STALE").expect("stale ref");

        let result = generate(&root, &sample_semantics()).expect("second run");

        assert!(!result.skill_written, "must not overwrite user skill");
        assert_eq!(fs::read_to_string(&skill_path).unwrap(), "USER EDITED");
        assert!(
            fs::read_to_string(&ref_path).unwrap().contains("wait"),
            "references must be regenerated"
        );
    }
}
```

- [ ] **Step 3: 运行确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ai_pack`
Expected: 编译错误(`generate` 未定义)。

- [ ] **Step 4: 实现(tests 上方)**

```rust
fn render_instructions_markdown(data: &EclMapSemanticData) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# th{} ECL 指令参考\n\n由 THTK-Studio 从 eclmap 自动生成,请勿手工编辑(会被刷新覆盖)。\n\n",
        data.version
    ));
    out.push_str("| opcode | 指令名 | 签名 | 参数 | 分组 |\n|---|---|---|---|---|\n");
    for ins in &data.instructions {
        let params = ins
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.type_name))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            ins.opcode,
            ins.name,
            ins.signature.as_deref().unwrap_or(""),
            params,
            ins.section.as_deref().unwrap_or("")
        ));
    }
    out
}

fn render_registers_markdown(data: &EclMapSemanticData) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# th{} 全局寄存器参考\n\n由 THTK-Studio 从 eclmap 自动生成,请勿手工编辑。\n\n",
        data.version
    ));
    out.push_str("| id | 名称 | 类型 |\n|---|---|---|\n");
    for global in &data.globals {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            global.id, global.name, global.var_type
        ));
    }
    out
}

/// 在项目根生成 .claude/skills/ecl-modding/:
/// SKILL.md 仅在缺失时写入(保护用户修改);references/ 总是刷新。
pub fn generate(project_root: &str, data: &EclMapSemanticData) -> Result<AiPackResult, String> {
    let skill_dir = Path::new(project_root).join(".claude/skills/ecl-modding");
    let references_dir = skill_dir.join("references");
    fs::create_dir_all(&references_dir)
        .map_err(|e| format!("Failed to create skill directories: {e}"))?;

    let instructions_file = format!("th{}-instructions.md", data.version);
    let registers_file = format!("th{}-registers.md", data.version);

    let mut reference_files = Vec::new();

    let instructions_path = references_dir.join(&instructions_file);
    fs::write(&instructions_path, render_instructions_markdown(data))
        .map_err(|e| format!("Failed to write instructions reference: {e}"))?;
    reference_files.push(instructions_path.to_string_lossy().to_string());

    let registers_path = references_dir.join(&registers_file);
    fs::write(&registers_path, render_registers_markdown(data))
        .map_err(|e| format!("Failed to write registers reference: {e}"))?;
    reference_files.push(registers_path.to_string_lossy().to_string());

    let skill_path = skill_dir.join("SKILL.md");
    let skill_written = if skill_path.exists() {
        false
    } else {
        let content = SKILL_TEMPLATE
            .replace("{{VERSION}}", &data.version)
            .replace("{{INSTRUCTIONS_FILE}}", &instructions_file)
            .replace("{{REGISTERS_FILE}}", &registers_file);
        fs::write(&skill_path, content).map_err(|e| format!("Failed to write SKILL.md: {e}"))?;
        true
    };

    Ok(AiPackResult {
        skill_path: skill_path.to_string_lossy().to_string(),
        skill_written,
        reference_files,
        version: data.version.clone(),
    })
}
```

`src-tauri/src/modules/ecl/mod.rs` 追加 `pub mod ai_pack;`。

- [ ] **Step 5: 运行确认通过**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ai_pack`
Expected: 2 passed。

- [ ] **Step 6: Tauri 命令 + 注册 + 前端入口**

1. `src-tauri/src/modules/ecl/commands.rs` 末尾追加:

```rust
#[tauri::command]
pub async fn generate_ai_assist_pack(
    state: State<'_, AppState>,
) -> Result<super::ai_pack::AiPackResult, String> {
    let config = state.config_manager.get_config();
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("No project root set")?;

    let map_path = crate::modules::mcp::tools::resolve_map_path(&config)?;
    let semantics = map_parser::parse_ecl_map_file(&map_path)?;
    super::ai_pack::generate(&root, &semantics)
}
```

2. `src-tauri/src/main.rs`:import 列表(line 22-25)加入 `generate_ai_assist_pack`,`generate_handler![` 中 `run_thecl_operation,` 后追加 `generate_ai_assist_pack,`。

3. `src/api/modules/eclMap.js` 追加:

```js
export function generateAiAssistPack() {
  return invoke('generate_ai_assist_pack')
}
```

(确认该文件已有 `import { invoke } from '@tauri-apps/api/core'`,没有则补。)

4. `src/components/Common/MenuBar.vue`:
   - `menus` 中"脚本"菜单(含 `script.compileEcl` 的那组)追加选项 `{ label: '生成 AI 辅助包', key: 'script.generateAiPack' }`
   - switch(`case 'script.generateHeader':` 之后)追加:

```js
    case 'script.generateAiPack': {
      const reportsStore = useWorkbenchReportsStore()
      generateAiAssistPack()
        .then((result) => {
          reportsStore.publishToolResult({
            ownerKey: 'ecl:ai-pack',
            source: 'toolchain',
            operation: 'ai-pack',
            scriptKind: 'ecl',
            title: '生成 AI 辅助包',
            path: result.skillPath,
            success: true,
            message: [
              result.skillWritten ? 'SKILL.md 已生成' : 'SKILL.md 已存在,保留用户版本',
              ...result.referenceFiles.map((file) => `已刷新 ${file}`)
            ].join('\n'),
            diagnostics: []
          })
          workbenchPanelsStore.showBottomPanel('output')
        })
        .catch((error) => {
          reportsStore.publishToolResult({
            ownerKey: 'ecl:ai-pack',
            source: 'toolchain',
            operation: 'ai-pack',
            scriptKind: 'ecl',
            title: '生成 AI 辅助包',
            path: null,
            success: false,
            message: String(error),
            diagnostics: []
          })
          workbenchPanelsStore.showBottomPanel('output')
        })
      break
    }
```

   顶部 import 补 `generateAiAssistPack`(from `'../../api'`)与 `useWorkbenchReportsStore`(确认是否已 import,没有则加)。

- [ ] **Step 7: 全量验证**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && npm run build`
Expected: Rust 全部测试通过 + 前端构建成功。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/assets/ src-tauri/src/modules/ecl/ src-tauri/src/main.rs \
  src/api/modules/eclMap.js src/components/Common/MenuBar.vue
git commit -m "feat: generate project-level ECL modding skill + eclmap references"
```

---

### Task 11: 文档更新 + 手动验收清单

**Files:**
- Modify: `editor-shell-status.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: 更新 editor-shell-status.md**

- §3.1("当前终端不是完整终端")整节替换为已实现说明:真 PTY 终端(portable-pty + xterm.js,多会话,scrollback 5000)
- §2 新增小节"2.8 Agent 通道":MCP server 六工具、.mcp.json 自动接线、AI 辅助包生成
- §6 风险点删除"真终端尚未实现"与"终端 store 无内存上限"两条

- [ ] **Step 2: 更新 CLAUDE.md**

"Backend structure" 一节:`terminal`(one-shot,已被 PTY 终端取代但保留)描述更新;`common/` 列表补 `pty`(PTY 会话)与 `mcp_config`(.mcp.json 合并);`modules/` 补 `mcp/`(rmcp server)。"Frontend structure" 的 `services/` 补 `terminal/sessionRuntime.js`。

- [ ] **Step 3: 手动验收清单(用户在带桌面的机器执行,对照 spec §7)**

```
1. npm run tauri dev 启动,打开一个项目文件夹
2. 底部终端面板:新建终端 → 跑 vim / python ── 交互正常、resize 正常
3. 项目根出现 .mcp.json,含 thtk-studio entry(端口 + Bearer token)
4. 终端里跑 claude → /mcp 应列出 thtk-studio 六个工具
5. 让 agent 调 check_ecl 编译一个 .decl ── 诊断与问题面板一致
6. 让 agent 调 report_to_user ── 输出面板弹出卡片
7. .mcp.json 中手写一个假 server entry,重开项目 ── entry 保留
8. 菜单"生成 AI 辅助包" ── .claude/skills/ecl-modding/ 生成;改 SKILL.md 后重跑不被覆盖
```

- [ ] **Step 4: Commit**

```bash
git add editor-shell-status.md CLAUDE.md
git commit -m "docs: record agent channel (PTY + MCP + AI pack) in status docs"
```

---

## 已知风险与对策

| 风险 | 对策 |
|---|---|
| rmcp 1.7 宏/路径与计划代码有出入 | 结构已对齐官方示例;编译失败时以 docs.rs/rmcp/1.7.0 修正 import,不改架构 |
| `AppConfig` 无 `Default` derive | Task 7 Step 2 注明:确认后补 derive |
| Windows ConPTY 行为差异 | portable-pty 封装;计划在用户 Windows 机器跑验收清单 2 |
| xterm 输出 UTF-8 跨 chunk 截断 | 已注释标记;出现乱码时改为 Rust 侧累积解码 |
| Linux 服务器无桌面,无法本机端到端验证 | 单测覆盖逻辑层;验收清单交用户执行 |
