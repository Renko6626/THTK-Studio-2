# truth 固定基线核对

日期：2026-11-09。仅验证原始基线，不代表 GBK 补丁或 Studio 集成已经完成。

## 来源与构建

- truth：zero318/truth，`c78a541d1fb09fcefd448c83b097d4f9b815fce3`，检出目录未修改源码和 Cargo.lock。
- maps：zero318/TouhouMaps，`0673f4221f5e975842162e07cd7b765fc7570ebd`。
- 平台：Linux x86_64。
- Rust 1.98.1、1.92.0：构建失败，原因是旧源码依赖 never-type fallback 的类型推断，被新编译器作为错误报告。
- Rust 1.85.1：成功。未通过关闭 lint 或修改上游源码绕过错误；保留上游已有警告。

实际成功命令（truth 检出目录）：

```bash
cargo +1.85.1 build --release --locked --bin truth-core --bin trumsg --bin trustd
./target/release/trumsg --version
./target/release/trustd --version
./target/release/trumsg compile --help
./target/release/trustd decompile --help
```

两工具版本输出均为 `truth c78a541`。help 确认 --ending、-m、-g、-o；无编码选项和定义 JSON 查询命令。三个可执行文件在同一目录运行；没有单独执行“删除 truth-core”验证。

## 代表性真实样本

输入来自本机 renkolab/local/th18.v1.00a/dat，只读；源资产和反编译文本不提交进 Studio 仓库。输出均写临时目录。

| 样本 | map | 反编译 | 原样回编 | 与原二进制比较 |
| --- | --- | --- | --- | --- |
| st01a.msg，stage | th18.msgm | exit 0 | exit 0 | 字节一致 |
| e01.msg，ending | th10.endm | exit 0 | exit 0 | 字节一致 |
| st01.std | th18.stdm | exit 0 | exit 0 | 字节一致 |

操作形状（示例路径是占位，实际 argv 见本机 probe-results.json）：

```bash
trumsg decompile INPUT.msg -g 18 -m MAP.msgm -o OUTPUT.msgspec
trumsg compile OUTPUT.msgspec -g 18 -o REBUILT.msg
trumsg decompile INPUT.msg -g 18 -m MAP.endm -o OUTPUT.endspec --ending
trumsg compile OUTPUT.endspec -g 18 -o REBUILT.msg --ending
trustd decompile INPUT.std -g 18 -m MAP.stdm -o OUTPUT.stdspec
trustd compile OUTPUT.stdspec -g 18 -o REBUILT.std
```

本机日志：`/tmp/thtk-map-audit-JoJOvc/build-rust-1.85.1.log`、同目录 `probe-results.json`；临时文件不作为长期依赖。

未验证：局部修改后效果、GBK、其他作品、Windows 构建/安装包、游戏运行。三份原样往返成功不等于完整迁移验收。

## 对后续计划的影响

- 后续 truth 基线构建优先固定 Rust 1.85.1，无须为了当前编译器兼容问题增加 truth 源码补丁。
- map 由 Studio 自行解析管理，不为查询维护 truth 分支或增加查询接口。
- GBK 仍按独立编码设计处理；未批准通过本次 map 决策扩展下游维护范围。

## Studio map 解析接入核对

新增显式 map 读取后，用 TH18 STD 反编译源码插入一次 `camera_position_interp(60, Linear, 0.0, 0.0, 0.0)`，truth 编译 exit 0；删掉最后一个参数后 exit 1，诊断为 expects 5 arguments, got 4。这与 Studio 对 `SU(enum="InterpMode")fff` 的 5 参数解析一致。输出在本机 `map-shape-probe.json`；仅验证参数形状，未验证修改后游戏效果。
