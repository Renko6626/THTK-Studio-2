# 脚本格式支持现状与扩展规划

日期:2026-06-07
作者:扩展前的架构盘点 + 旧 PyQt 版 THTK-Studio 参考挖矿

## 1. 现状(谁是一等公民,谁还是占位)

| 工具/格式 | 后端解析+诊断 | Monaco 语法/补全 | 工作区视图 | 构建对话框 | MCP 工具 |
|---|---|---|---|---|---|
| **thecl / .decl** | ✅ `modules/ecl/` 完整 | ✅ `services/languages/ecl/` 全套 | ✅ 文本 + `binary-script` | ✅ | ✅ check/compile/decompile/lookup |
| **thmsg / .msg** | ✅ 基本反编译/编译 + dmsg 翻译层(`modules/msg/`) | ❌ 无 Monaco 语言服务 | ✅ 基本可用 — `.msg` 进 binary-script 视图，`.dmsg` 进普通文本视图 | ❌ 菜单触发,无 BuildDialog | ❌ 无 MCP 工具 |
| **thanm / .anm** | ❌ | ❌ | ❌ | ❌ stub | ❌ |
| **thstd / .std** | ✅ 基本反编译/编译 + dstd 翻译层(`modules/thstd/`,含 jmp opcode 1 参数交换) | ❌ 无 Monaco 语言服务 | ✅ 基本可用 — `.std` 进 binary-script 视图，`.dstd` 进普通文本视图 | ❌ 菜单触发,无 BuildDialog | ❌ 无 MCP 工具 |
| **thdat / .dat** | ✅ 基本解包/打包(`modules/thdat/`,无语义层) | — | ✅ 通过菜单 + native pickers | ❌ 无 BuildDialog | ❌ 无 MCP 工具 |

`services/toolchains/registry.js` 里 thmsg/thanm/thstd/thdat 都有 descriptor,`supportsBuildDialog: false`——空架子搭好了,等填。

ECL 那条线证明了**目标分层是对的**:
- Rust `modules/ecl/` 完成 thecl 调用 / stderr→结构化诊断 / eclmap 语义解析
- 前端 `services/languages/ecl/` 把语义数据接入 Monaco(补全/悬停/跳转/引用/大纲/诊断)
- `services/workbench/editorViews.js` 注册视图类型,`services/toolchains/registry.js` 注册工具链
- MCP 把这些能力转给 agent

新工具链照葫芦画瓢,**架构本身不需要改**。

## 2. 旧 PyQt 版本可借鉴的东西(直接搬运/移植)

旧仓库:`https://github.com/Renko6626/THTK-Studio/`(已克隆到 `/tmp/thtk-old-ref/THTK-Studio/`,gitignore 已记)。

### 2.1 ★ 语义参考表(最有价值)

旧仓库 `resources/` 里有社区/作者自己整理的中文语义 JSON,**thtk 官方 eclmap 不覆盖这些**:

| 文件 | 内容 | 大小 | 直接用法 |
|---|---|---|---|
| `thmsg_ref.json` | msg opcode → `[signature, 中文描述]` | 完整 | 当作 thmsg 的"伪 eclmap",驱动 Monaco 补全/悬停 |
| `thstd_ref.json` | std opcode → `[signature, 中文描述]` | 完整 | 同上 |
| `anm_syntax_definitions.json` | anm 语法关键字定义 | — | anm 语法高亮的种子 |
| `default.anmm` | anmm 映射文件(thanm 自己的 map 格式) | — | thanm 的 eclmap 对应物 |

格式样例(thmsg_ref):
```json
{ "0": ["end()", "完全终止脚本引擎的运行。"],
  "1": ["playerShow(int who)", "显示玩家的角色立绘。"],
  "3": ["textboxShow()", "显示对话框。"] }
```

签名里直接含参数类型 + 名字,转 Rust `EclMapInstructionSpec`-like 结构毫无障碍。**抄完省下数月研究**。

### 2.2 thmsg 的"两段反编译"流程(必学,否则用户拿不到可读文本)

`thmsg -d` 产出的是 `.dmsg` 二进制中间文本,长这样:
```
0:ins_3()
0:ins_1(0)
```

旧 wrapper 的 `_translate_dmsg_to_txt` 用 ref json **把 ins_N 翻译成 textboxShow() 这样**写入 `.txt`。打包反向走 `_recover_txt_to_dmsg` 再调 `thmsg -c`。

ECL 没有这步(thecl 直接给文本)。**新版的 thmsg 工作流必须实现这一层**,否则用户编辑的是不可读的 ins_N。

### 2.3 ANM 的精灵预览(领域专有 IDE 价值)

旧仓库 `app/widgets/sprite_preview.py` + `sprite_composer.py` + `core/image_manager.py`:
- thanm 解包出 spritesheet PNG + 文本脚本
- 脚本里有 sprite 矩形(`{x, y, w, h}`)
- IDE 把 spritesheet 裁切显示;脚本编辑联动高亮

这是 ANM 没法用纯文本 IDE 替代的关键——人眼看不出"sprite #42 是个什么图"。**这是 ANM 工具链做完后,工作流闭环的最后一公里**。

后端职责(应放进 Rust `modules/anm/`):
- 调 thanm 解包(产出 spritesheet + 脚本)
- 解析脚本提取 sprite/anim 元数据
- 给前端返回 `{ sprites: [{id, rect, file}], anims: [...] }` 结构化数据

前端职责(`components/Editor/AnmPreviewView.vue` + `composables/useSpriteAtlas.js`):
- Canvas/CSS 裁切 spritesheet 显示 sprite 网格
- 选中脚本里的 sprite 引用 → 预览高亮 / 反向

### 2.4 旧仓库的"处理器驱动"模式

旧 `app/handlers/{ecl,msg,anm,std}_handler.py` + 共享 `ScriptHandler` 接口——本质就是我们现在 `services/toolchains/registry.js`(descriptor) + `services/workbench/editorViews.js`(view) 这套注册表机制的等价物。**架构已经对位,不用学旧的**。

值得偷的是**职责拆分粒度**:旧版每个工具有 `wrapper`(进程封装)/ `handler`(协调)/ `panel`(UI)/ `syntax_highlighter`(高亮)四件套——对应到我们就是 `modules/{tool}/`(后端)+ `services/languages/{tool}/`(前端语言服务)+ `components/Editor/{Tool}View.vue`(视图,如果需要专用)+ registry descriptor。

## 3. msg / std / anm 扩展规划(粗粒度)

按"先简单后复杂、先用户痛点最大"排序:

### 阶段 A:thmsg(对话脚本,最简单 + 用户高频)

工作量估计:**接近 thecl 的 60%**(无 sprite,无 anmm 映射,但多一层 dmsg↔txt 翻译)。

Rust(`modules/msg/`):
- `compiler.rs` 调 thmsg -d / -c
- `translator.rs` 实现 dmsg ↔ txt(用 ref json,opcode 名 + 注释)
- `map_parser.rs` 读 `thmsg_ref.json`(从旧仓库移植 + 转结构)

前端(`services/languages/msg/`):
- 语法高亮(Monarch)
- 补全/悬停(走 ref json,与 ECL 同模)
- 编辑视图复用 Monaco 文本视图(扩展 `.dmsg`)

注册:registry.js 的 `thmsg` descriptor 填实;editorViews.js 加 `.dmsg` → text 映射;MCP 加 `check_msg`/`compile_msg`/`decompile_msg`/`lookup_msg_semantics`(套 ECL 现有的可测自由函数模式)。

资产:**直接复制 `thmsg_ref.json` 到 `src-tauri/assets/`**(已是公共数据格式,不引入许可冲突)。

**已完成(2026-06-07)**：

- `src-tauri/assets/msg-th17.json`：33 条指令的 JSON 语义数据（从旧仓库 `thmsg_ref.json` 转换，规范化 schema）
- `src-tauri/src/modules/msg/`：`map_parser`（反序列化 + th17 内嵌回退）、`translator`（行级 `ins_N` ↔ 指令名双向翻译；时间标签/注释/未知 opcode 透传）、`compiler`（thmsg -d/-c 调用 + SJIS↔UTF-8 桥接）、`commands`（两个 Tauri 命令：`decompile_msg_file` / `compile_msg_file`）
- 前端菜单"脚本 → 解包当前 .msg / 打包当前 .dmsg"：从活动标签取路径，成功/失败均发输出面板卡片，解包后自动打开生成的 .dmsg
- Windows 验收清单：见 `editor-shell-status.md` §9 条目 14–17

**已知限制**：`.msg` 文件的 binary-script 视图目前使用 ECL 专用文案（`BinaryScriptView.vue` 写死了"打开 ECL build dialog"按钮），对 .msg 无意义。thmsg 工作流仍可正常使用（从顶部菜单触发），但视图内的操作按钮属于后续 UX 任务：让 binary-script 视图变成工具感知，或为 .msg 单独注册一种视图类型。

### 阶段 B:thstd(3D 背景脚本,与 msg 同形)

工作量估计:**与 thmsg 几乎相同**——格式更简单,没有 dmsg 中间层(thstd 直接吐文本);只要复制 thmsg 模块改名 + 换 ref json(`thstd_ref.json`)。

**已完成(2026-06-07)**：

- `src-tauri/assets/std-th17.json`：19 条指令的 JSON 语义数据（从旧仓库 `thstd_ref.json` 转换；跳过 3 条 ins_N 占位）
- `src-tauri/src/modules/thstd/`：`map_parser`（反序列化 + th17 内嵌回退）、`translator`（行级 `ins_N` ↔ 指令名双向翻译；**opcode 1 (jmp) 参数交换 quirk**——thstd 二进制 ins_1(offset,time) ↔ ref/ECL 生态 jmp(time,offset)）、`compiler`（thstd -d/-c，UTF-8 全程，无 SJIS 桥接，比 thmsg 简单一层）、`commands`
- 前端菜单"脚本 → 解包当前 .std / 打包当前 .dstd"——解包后 auto-open .dstd
- Windows 验收清单：见 `editor-shell-status.md` §9 条目 18–21

**暂缓**：`@label` → 字节 offset 预处理（jmp 跳到 label 而不是字节）由用户决定封装到下一版 thtk，本期不实现。

### 阶段 C:thanm(精灵动画,最重 — 因为要做预览)

工作量估计:**与 thecl 全等甚至更多**,因为多了图像 pipeline:
- 文本编辑层:同 msg/std 模式(thanm 文本 + anmm map)
- **领域预览层**:spritesheet 加载 + 裁切 + Canvas 渲染 + 脚本联动(新增 ~3 个组件 + 1 个 composable + Rust 图像元数据提取)

可以**先做文本编辑层(短半工作量出活)**,预览层单独排一个 Spec。

### 阶段 D:thdat(打包工具,非脚本)

**已完成(2026-06-07)**：

- `src-tauri/src/modules/thdat/`：`compiler`（extract `-xd` 自动检测版本 / pack `-c{ver}` 取 `effective_thdat_version`）、`commands`（`extract_dat_file` / `pack_dat_file`）。**无 map_parser/translator**——纯容器管理。
- 命令行长度上限 28KB 守卫（thdat pack 必须列所有文件名作为参数）；超限返回 Err 不调 thdat
- 前端菜单"脚本 → 解包 .dat / 打包目录为 .dat"：`@tauri-apps/plugin-dialog` 的 `open`/`save` 选源/目标，完成后 `projectStore.refresh()` 刷新文件树
- 操作不依赖 activeTab（thdat 操作的是容器，不是当前打开的脚本）
- Windows 验收清单：见 `editor-shell-status.md` §9 条目 22–25

**已知限制**：目前 pack 不递归子目录（只打包顶层文件）——thdat 的 .dat 本来也是平坦结构，但如果用户的源目录有嵌套，会被忽略。未来若发现 thtk 支持嵌套打包再扩展。

## 4. 落地建议(给下一次开发会话)

1. **从 thmsg 开始**:小 + 用户痛点高(对话翻译/汉化是主要 modding 场景之一) + 把 ECL 的可复用框架真正测出来(translator 这层 ECL 没有,挑战架构)。
2. **复用 ECL 的 MCP 工具模式**:`modules/mcp/tools.rs` 里现成的"effective_toolchain_config + spawn_blocking + tool_error"模板,msg/std/anm 各加 4-5 个工具是机械活。
3. **不要急着搞 anm 预览**:文本编辑层 + ref 语义先打通,预览作为后续 Spec(它本身就值得一个独立 brainstorming——VS Code 内联图片 vs 独立面板 vs Webview 嵌 Canvas)。
4. **eclmap 那条 AI 辅助包路径要泛化**:`generate_ai_assist_pack` 应能对 msg/std/anm 也生成参考表(从 ref json),不再只是 ECL 专属——这是 AI agent 帮人写 msg 脚本的关键。

## 5. 不在本轮范围

- **sht(Shoot Type)文件**:用户之前提过想研究 ExpHp/Priw8 的发现。它本质是配置而非脚本,**不走 thtk 工具链**(thtk 不处理 sht),不在 msg/anm/std 扩展范围。需要时单独走研究 + 设计流程。
- ANM 预览(留到阶段 C 的后半部分单独 spec)。
- 字符串提取/批量翻译工具(汉化场景)——独立功能,不在脚本编辑范围。
