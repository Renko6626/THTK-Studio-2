# ECL 映射研究状态

最后核对：2026-11-22。TH16–18 已达到原版 ECL 可用线。三作各检查 22 份零售版 ECL，原版出现的不同 opcode 中，TH16 为 177/177、TH17 为 154/154、TH18 为 164/164 达标。Studio 自带的
ECL map 仍来自固定的 zero318 快照；修订后的指令参考和 thecl 配置在独立资料库维护。

在本地资料库中可查 [TH16 逐项参考](../../../TouhouMaps/docs/ecl/th16/th16-zero318-style.md)、
[TH17 双名称参考](../../../TouhouMaps/docs/ecl/th17/th17-instructions.md)、
[TH18 双名称参考](../../../TouhouMaps/docs/ecl/th18/th18-instructions.md)。
[使用说明](../../../TouhouMaps/README.md)给出了两套拼法和编译命令。
这些相对链接适用于 Studio 与 TouhouMaps 并排检出的本地目录。

[可靠度 CSV](../../../TouhouMaps/docs/ecl/th16-th18-instruction-reliability.csv)按作品和 opcode 归档。原版使用的每个编号都已核查或可相对可靠地继承；其余 66 条待核记录涉及 27 个编号，留待后续按需核查。可从[待核分组与证据](../../../TouhouMaps/docs/research-handoff-2026-11-10.md#跨作继承与待核顺序)选题，或用[检查脚本](../../../TouhouMaps/scripts/check_stock_ecl.py)重新核对原版覆盖。

完整交接保存在独立资料库：

- [交接与研究入口](https://github.com/Renko6626/TouhouMaps/blob/studio-ecl/docs/research-handoff-2026-11-10.md)
- 分叉 `Renko6626/TouhouMaps`，维护分支 `studio-ecl`。
- 初始资料提交 `39a2956dd99f0a455e521a3832229234ed45e9d1`。
- 本地位置 `/data/sunyunbo/www/TouhouMaps`。

11 月 10 日的初始导入建立了共用签名和两套名称目录。22 个版本都有文件、44 种
组合能被 thtk 加载，不代表各作实际指令和语义已确认。记录中的 228 条签名差异
主要比较 zero318 `_thtk` 文件与 thecl 内置表；6 条变量类型差异集中在 `-9961`。
3,789 条中性名称包含风格缺名和编译器继承条目，不能直接当作待逆向规模。

初始导入的“默认采用 thecl 内置签名”“所有双下划线名称不参与对应”等取舍
尚未获得游戏语义验证。继续其他作品时先看完整交接中的修正说明，不能继续把这些
导入规则当作事实。完整明细已随交接持久保存。

当前不替换 Studio 默认 ECL map，不扩大自动名称归并，不批量转换用户源码。
`deps/manifest.source.json` 与产品 map 保持现有来源；研究其他作品前先核对
目标游戏版本、样本 hash、已有持久工程和参数读取／消费者路径。
