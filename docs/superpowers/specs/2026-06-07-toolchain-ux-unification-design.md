# 工具链前端 UX 统一(thecl / thmsg / thstd / thdat)

日期:2026-06-07 状态:已批准范围
触发:四工具链落地后审阅,发现菜单标签/动词/卡片标题/代码结构都不统一

## 关键决策(锁定)

### 1. 动词

- **thecl / thmsg / thstd**:统一用 **"反编译 / 编译"**(技术准确,与 thtk 文档一致)
- **thdat**:**"解包 / 打包"**(容器语义)

### 2. 菜单标签

统一格式:**`<动词>当前 .<扩展> (.ext → .ext)`**(类似 .ecl ↔ .decl 提示),例:

| 工具 | 反编译 / 解包 | 编译 / 打包 |
|---|---|---|
| thecl | 反编译当前 .ecl | 编译当前 .decl |
| thmsg | 反编译当前 .msg | 编译当前 .dmsg |
| thstd | 反编译当前 .std | 编译当前 .dstd |
| thdat | 解包当前 .dat | 打包目录为 .dat |

ECL 高级选项:`反编译当前 .ecl(高级选项…)` 仍走 BuildConfigDialog
ECL 头文件:`生成 ECL 头文件…` 不变(本来就是高级活)

### 3. BinaryScriptView 描述模板

统一格式:
- **typeLabel**:`Touhou <NAME> 二进制` / `Touhou DAT 容器`
- **description**:一句性质描述,**主动语态**
- **suggestion**:一句操作建议
- **actionLabel**:`反编译为 .<ext>` / `编译为 .<ext>` / `解包到目录…`

具体见各 descriptor。

### 4. 菜单分组(divider)

```
ECL 反编译/编译
ECL 头文件生成
---
MSG 反编译/编译
---
STD 反编译/编译
---
DAT 解包/打包
---
AI 辅助包生成
```

### 5. 统一 publish helper

新建 `src/composables/useToolchainResult.js` 暴露 `publishToolchainResult({ tool, operation, inputPath, outputPath, success, message, extra })`:
- 内部表查 `<动词>` + `<扩展>` 算 title
- 自动 `panelsStore.showBottomPanel('output')`
- 旧的 4 个 publish helper(`publishMsgResult` / `publishStdResult` / `publishDatResult` / BinaryScriptView 内 `publishView`)全部删除替换

### 6. 统一 action composable

新建 `src/composables/useToolchainActions.js` 暴露:
- `runDecompileEclQuick(path)` / `runDecompileEclAdvanced(path)`(打开 BuildDialog)/ `runCompileEclQuick(path)` / `runGenerateEclHeader(path)`(BuildDialog)
- `runDecompileMsg(path)` / `runCompileMsg(path)`
- `runDecompileStd(path)` / `runCompileStd(path)`
- `runExtractDat(path)` / `runPackDat()`

每个 run* 内部:
1. 校验输入(空路径 / 错扩展)
2. dirty tab 提示保存(已实现于 compile 路径)
3. 调 API
4. 调 publishToolchainResult
5. auto-open .decl/.dmsg/.dstd / refresh tree (dat)
6. 错误进 catch 也发 publishToolchainResult

MenuBar 和 BinaryScriptView 都用这套——**两处对照 ECL 单源**。

### 7. ECL 快速路径

菜单 / BinaryScriptView 默认走"快速反编译":
- 版本 = `effective_thecl_version`(同 thmsg/thstd 模式,project_config 覆盖 app config)
- mapPaths = effective(从 project_config / app config / 默认 eclmap_path)
- useShiftJis = 走 project_config 的 encoding(默认 shift-jis)

直接复用现有 Rust `decompile_ecl_file(source_path, map_paths=[])` 命令(它已经走 effective 路径)。

BuildConfigDialog 保留,仅在用户明确"高级选项"时打开。

## 任务拆分

### Task 1 — composable + helper(新文件,不冲突)
- 新 `src/composables/useToolchainResult.js`
- 新 `src/composables/useToolchainActions.js`
- 不删旧 helper,只新增

### Task 2 — BinaryScriptView 切换 + 文案统一
- 替换内部 `publishView` 调用为新 composable
- 重写 `TOOL_DESCRIPTORS` 文案,所有 5 个 entry(含 anm disabled)
- typeLabel/description/suggestion/actionLabel 全部按新模板

### Task 3 — MenuBar 切换 + 菜单标签统一 + 分组 + ECL 快速路径
- 删除 `publishMsgResult/publishStdResult/publishDatResult/runDecompileMsg/runCompileMsg/runDecompileStd/runCompileStd/runExtractDat/runPackDat`(老的)
- 全部走 `useToolchainActions`
- 菜单标签按新格式;divider 分组
- 加新菜单项:`反编译当前 .ecl`(快速)+ `反编译当前 .ecl(高级选项…)`(BuildDialog)
- 保留 `编译当前 .decl`(走 BuildDialog,因 thecl compile 要选游戏版本)

注意:thecl compile 实际**用户必须选游戏版本**(thecl -c 必填),所以 compile 仍然走 BuildDialog 是合理的;**只有 decompile 可以快速化**(thecl -d 也需要版本,但可以从 project_config 取)。
**决策**:thecl compile 也试着走 quick(从 effective_thecl_version 取),BuildDialog 改成"反编译/编译(高级…)"的可选入口。BuildConfigDialog 表单本身不删。

### Task 4 — 验证 + 文档
- npm run build / cargo test
- 更新 editor-shell-status.md
- 更新 docs/script-support-status.md 表格(措辞统一后表格里的描述也跟着改)

## 不在范围

- BuildConfigDialog 删除(保留作为"高级选项"入口,**虽然多数用户不会用**——这是未来 Spec)
- AI 辅助包菜单的措辞(本来就独立的功能项)
- 右键菜单的对应入口(目前文件树右键已经支持 file 操作,工具操作只在顶部菜单 + binary view;**暂不在右键加**,留给后续)
- 工具链状态的视觉指示(目前 `disabled` 静默,可以后续加 hint 文字)

## 验收(Windows 单独跑)

1. 打开 .ecl/.msg/.std/.dat 文件,binary script view 显示统一风格的描述/建议/按钮
2. 菜单"脚本"下分组清晰,标签格式一致
3. 菜单触发"反编译当前 .ecl"快速反编译产出 .decl,无对话框
4. 菜单"反编译当前 .ecl(高级选项…)"仍弹 BuildDialog
5. 卡片标题:"反编译 .ecl 完成" / "反编译 .msg 完成" / "反编译 .std 完成" / "解包 .dat 完成"——格式一致
6. 失败时,卡片标题 "...失败",输出面板自动弹出

## 实际落地(2026-06-07)

| Task | Commit | 内容 |
|---|---|---|
| 1 | d9e342b | 统一 publish + actions composable(纯新增) |
| 2 | 6ba8405 | BinaryScriptView 切换 + 文案模板统一 + ECL 高级链接 |
| 3 | 494855f | MenuBar 菜单分组 + 标签统一 + ECL quick path(MenuBar 净减 264 行) |
| 4 | (此提交) | 验证 + 文档 |

总改动:
- 新增 2 个 composable(`useToolchainResult` / `useToolchainActions`)
- 替换 4 个 publish helper → 1 个
- MenuBar 工具链相关代码 ~316 行 → 52 行
- 菜单 8 项 → 12 项(ECL 加 quick / advanced 各 2 + 头文件;dividers 分组)
- BinaryScriptView 文案全部按统一模板
