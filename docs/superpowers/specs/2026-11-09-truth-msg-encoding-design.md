# truth MSG/END 游戏文本编码扩展

日期：2026-11-09。状态：最小编码补丁已实施，stage/ending GBK 字节及往返检查通过。补丁保存在 Studio 依赖目录，未创建额外上游分支或发布。

## 需求

Studio 现有工作流允许 Shift-JIS 解包日文原版，编辑 UTF-8 源码，再按 GBK 打包用于已适配的汉化游戏。迁移 truth 后保留此行为；解包和打包的编码独立选择。普通原作仍默认 Shift-JIS。

编码指游戏二进制中字符串的字符编码，不是源码文件编码，也不是指令的 XOR、padding 或 furigana 布局。

## 已核对的源码

基线为 zero318/truth `c78a541d1fb09fcefd448c83b097d4f9b815fce3`：

- `src/io.rs`：`Encoding` 已是 encoding_rs 类型，`DEFAULT_ENCODING` 是 SHIFT_JIS。
- `src/llir/lower.rs`：字符串编译使用 DEFAULT_ENCODING。
- `src/llir/raise/early.rs`：字符串反编译使用 DEFAULT_ENCODING。
- `src/cli_def.rs`：trumsg compile/decompile 没有现成编码参数。
- `src/llir/abi.rs`：字符串 mask、block size、furibug 等属于 ABI，必须保留。

## 方案比较

1. **推荐：给 trumsg 增加二进制字符串编码选项。** 复用现有 encoding_rs，沿当前编译/反编译上下文传递；只在 MSG/END 字符串边界使用。
2. Studio 在外部转码二进制：需要理解并重写字符串掩码、长度和 padding，越过编解码职责，不采用。
3. 保留 thmsg 处理 GBK：违背“MSG/STD 完整迁移、不留旧后端”的已确认目标，不采用。

## 推荐接口与行为

新增 CLI 参数建议：`trumsg compile|decompile --encoding shift-jis|gbk|utf-8`。默认 shift-jis；源码和 map 始终 UTF-8。显式 utf-8 保留当前项目配置能力，不作为原作推荐编码。

只对 Stage/Ending 有效；mission 模式不在本次范围，不能因为共享参数而暗中扩展其行为。STD 的资源路径、ECL、ANM 和其他工具继续默认编码。

推荐在请求/编译上下文中携带编码，按 LanguageKey::Msg/End 决定是否使用。不得把进程全局 DEFAULT_ENCODING 改成 GBK，不引入全局可变编码状态。

编译顺序：UTF-8 源字符串 → 所选字符编码 → 现有 ABI 字符串布局/掩码 → 二进制。反编译逆向执行。编码失败应返回清楚的诊断，不用替换字符静默丢字。

Studio 请求优先级沿用：本次显式编码 > 项目 encoding > shift-jis。构建表单和 MCP 使用同一规则。

## 补丁与分发

代码修改在 truth 独立检出目录进行，文件所有权局限于 CLI、上下文传递和字符串读写入口及必要测试。以可复现补丁保存在 Studio `deps/truth/patches/`，记录基线与补丁 SHA-256；具体补丁文件在实现后创建，不能只在某台机器的 clone 中修改。

使用补丁后的固定构建随包分发。回馈上游可以后续单独进行，本次没有授权自动提交 issue、PR 或发布外部产物。

## 验证

复用 truth 的集成测试基础，只补关键缺口：

- 一个 GBK 可表达、Shift-JIS 不可表达的文本样本，分别验证 编译/反编译；预期字节由独立编码依据固定，不能复制生产算法当 oracle。
- 同一 UTF-8 源文件按 Shift-JIS 和 GBK 分别编译得到正确内容，且切换不影响另一次调用。
- 关卡 MSG 与 END 都经过真实字符串编解码；至少覆盖一个现有 masked string 用例，复用已有 padding/furigana 用例。
- 不可编码字符返回错误，不静默替换；默认 Shift-JIS 行为不变。

先跑受影响用例，再运行 truth 相应 MSG/END 回归。无需因参数增加而穷举全部作品。Studio 集成层只验证配置传递与错误展示，不复制底层编码测试。

## 实施前确认

确认此方案后再修改 truth。若现有架构需要超出上述范围的重构，先报告具体障碍与替代方案；GBK 不能以“后续再补”的方式从最终迁移验收中移除。
