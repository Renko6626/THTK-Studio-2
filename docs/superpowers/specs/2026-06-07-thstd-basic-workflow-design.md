# thstd 基本工作流 + dmsg 翻译层(沿用 thmsg 模板)

日期:2026-06-07 状态:已批准范围

## 目标

让 .std 文件能在 IDE 内"解包→读名→编辑→打包"闭环。**不做花活**(同 thmsg)。

## 与 thmsg 的差异(实现关键)

| 维度 | thmsg | thstd |
|---|---|---|
| 解包输出 | stdout(SJIS bytes) | **文件**(第三个 CLI 参数,UTF-8) |
| 编码 | SJIS ↔ UTF-8 桥接 | **全程 UTF-8**(thstd 文件无日文) |
| 参数顺序特例 | 无 | **opcode 1 (jmp)** 参数顺序与 ref 不一致,翻译两侧需交换 |
| label → byte offset 预处理 | — | **暂缓**(用户打算封装到下一版 thtk) |
| ref 规模 | 36 条(33 入库) | 22 条(19 入库) |

## 在范围

1. **种子 `src-tauri/assets/std-th17.json`**:同 schema(MsgSemanticData 复用,只是 `tool: "thstd"`),从旧 `thstd_ref.json` 转换,跳过 ins_N 占位
2. **Rust `modules/std/`**:
   - `map_parser.rs`:逻辑结构同 `msg/map_parser.rs`,种子内嵌(include_str! `std-th17.json`)
   - `translator.rs`:**复用 thmsg translator 的行级正则机制**,加 opcode 1 特例:
     - 解包(dstd→readable):`ins_1(a, b)` → `jmp(b, a)`(参数交换)
     - 打包(readable→dstd):`jmp(a, b)` → `ins_1(b, a)`
   - `compiler.rs`:thstd -d/-c,**比 thmsg 简单**——`-d` 写文件,不需要 stdout bytes 捕获,不需要 SJIS 桥接
   - `commands.rs`:`decompile_std_file` / `compile_std_file`
3. **文件命名**(对齐 thmsg 模型):`.std` 二进制,`.dstd` 可读源码
4. **前端最小集成**:
   - api 加 `decompileStdFile` / `compileStdFile`
   - 菜单"脚本"加"解包当前 .std / 打包当前 .dstd",同 thmsg 模式(根据 activeTab 后缀启用/禁用)
   - 解包后自动打开生成的 .dstd
5. registry.js thstd descriptor 保留 `supportsBuildDialog: false`

## 不在范围

- `@label` → 字节 offset 预处理(用户暂缓,等下一版 thtk)
- Monaco 语法/补全/悬停、MCP 工具、AI 辅助包扩展、structured diagnostics、BuildDialog(同 thmsg)

## Translator opcode 1 特例(实现细节)

translator 主路径:正则识别 `ins_N` 或 `name`,查 map,替换。**在替换之后**对 opcode 1 再做参数交换:

```rust
// 解包侧 dmsg_to_readable:
//   解析出 opcode = 1 时,在替换 name 后,把 args 内的前两个逗号分隔实参互换
//   只交换前两个,后面的(如果有)原样保留
// 打包侧 readable_to_dmsg:
//   识别出 name == "jmp" 时,替换 ins_1 后同样交换前两个实参
```

参数交换函数实现:
```rust
fn swap_first_two_args(args: &str) -> String {
    let parts: Vec<&str> = args.splitn(3, ',').collect();
    match parts.as_slice() {
        [a, b] => format!("{},{}", b.trim_start(), a),
        [a, b, rest] => format!("{},{},{}", b.trim_start(), a, rest),
        _ => args.to_string(), // 0 或 1 个参数,无法交换,原样返回
    }
}
```
保留原参数的空格/格式细节,避免轻微抖动。

## 任务拆分(3 个,比 thmsg 紧凑)

### Task 1 — backend 全套(TDD)
- `assets/std-th17.json`(从 thstd_ref 转换,sort by opcode)
- `modules/std/{mod, map_parser, translator, compiler, commands}.rs`
- `main.rs` 注册两个命令
- 测试:map_parser(2:种子可读 + 未知版本回退)+ translator(8:基本翻译 + jmp 双向交换 + 多参数 jmp + 未知 opcode 透传 + 注释剥离 + 时间/段标签 SCRIPT: 透传 + 往返恒等 + 0参 jmp 不崩)+ compiler(4:infer 输出路径 .std↔.dstd / 版本归一 / 未配置错误 / serde 默认)

### Task 2 — 前端 + 验证
- api 包装 + 菜单两条 + activeTab 后缀判断
- auto-open .dstd 同 thmsg
- npm run build + cargo test 全套通过

### Task 3 — 文档
- script-support-status.md thstd 行 ✅;CLAUDE.md backend 加 modules/std;editor-shell-status.md 加验收清单条目 18-21

## 验收(Windows 单独跑)

18. 菜单"脚本 → 解包当前 .std":选中 .std 标签后该项可用;触发后输出面板"解包 .std 完成",自动打开 .dstd
19. 编辑 .dstd 中的 jmp,验证:`jmp(60, 100)` 打包后 thstd 接受(参数顺序对得上 thstd 二进制约定)
20. 故意写错(`fakeFn()`)打包失败卡片显示 thstd stderr,.std 不被覆盖
21. thtk_dir 未配置:菜单触发立刻失败 "thstd path is not configured"
