# TH13–20 ECL 指令参考

Studio 按作品和 opcode 查询指令。编辑器中可以把鼠标停在当前 map 的指令名或 `ins_编号` 上，查看本作评级、参考签名和已有说明。MCP 工具 `lookup_ecl_semantics` 支持按编号或名称查询。例如，传入 `{"game_version":"18","query":"1001"}` 可查 TH18 的 1001 号，研究结果在 `instructionReferences` 中。项目 map 给出的名称、参数和签名仍用于编辑和编译。逐作研究数据随 Studio 分发，查阅时无需检出研究仓库。[内置数据](../../src-tauri/assets/ecl-reference/)

## 覆盖与待核项

“已核查”表示有本作证据，包括已确认直接返回的无行为指令。“相对可靠地继承”表示来源一致或 zero318 对用途有明确说明。“需要进一步核查”表示来源冲突、用途不明或仅有编译器回退等证据。原版可用线要求每个在本作零售 ECL 出现的 opcode 至少达到“相对可靠地继承”。

| 作品 | 收录 opcode | 已核查 | 相对可靠地继承 | 需要进一步核查 | 达标 / 原版出现的 opcode |
| --- | ---: | ---: | ---: | ---: | ---: |
| TH13 | 272 | 216 | 44 | 12 | 177 / 177 |
| TH14 | 290 | 229 | 49 | 12 | 176 / 176 |
| TH15 | 291 | 52 | 220 | 19 | 169 / 169 |
| TH16 | 295 | 131 | 146 | 18 | 177 / 177 |
| TH17 | 295 | 52 | 225 | 18 | 154 / 154 |
| TH18 | 298 | 177 | 101 | 20 | 164 / 164 |
| TH19 | 435 | 0 | 250 | 185 | 未评估 |
| TH20 | 435 | 0 | 254 | 181 | 未评估 |

TH13–18 共检查 134 份原版 ECL，剩余 99 条待核记录均未在对应样本中出现。新脚本用到待核项时优先看本作证据，尤其注意同编号的作品专属机制。TH19、TH20 暂时继承 zero318 各自的正式版定义，已知疑点随条目标出。本作逆向与原版覆盖留待以后。

## 数据维护

评级、两套研究名称与逐条证据集中在 [TouhouMaps 总表](../../../TouhouMaps/data/ecl/instructions.csv)。总表保留完整双风格名称；Studio 随包提供可独立归因的拼法。项目自带的 map 也能提供查询别名。上游版本与归属见[来源说明](../../../TouhouMaps/SOURCES.md)。内置签名表的 [thtk 许可](../../src-tauri/assets/ecl-reference/thtk.COPYING)随应用提供。研究仓库使用 `Renko6626/TouhouMaps` 的 `studio-ecl` 分支。相对链接和以下命令假定两个仓库并排检出。

在 TouhouMaps 根目录运行以下命令更新八作的 Studio 快照。加上 `--check` 可以检查已分发的数据与总表是否一致。

```bash
python3 scripts/export_ecl_reference.py --output-dir ../THTK-Studio-2/src-tauri/assets/ecl-reference
```

原版可用线由[检查脚本](../../../TouhouMaps/scripts/check_stock_ecl.py)读取同一张总表核对。后续逆向优先复用 renkolab 中对应版本的 Ghidra 工程、th-re-data 符号和既有标注。默认编译 map 继续使用固定的 zero318 快照，切换来源单独评估。
