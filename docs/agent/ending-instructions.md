# END 指令参考

人工维护的结局／staff 指令说明，文件结构和例子见 [END 编程介绍](ending.md)。支持 TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH18、TH19、TH20；TH06–TH09 的旧结局不使用这套接口。END 编号独立于关卡 MSG。

按当前任务的目标版本选择明确列出的行；每行独立，不把一个版本的分页、等待、颜色或 staff 规则推广给其他版本。项目默认版本和二进制文本编码仅供参考，源码保持 UTF-8。只确认签名而未整理运行行为的部分在对应行中说明。

调用采用随包默认 map 的名称，参数按列出顺序传入。int、float、string 为源码类型；项目 map 可改名或签名，遇到不一致时用现有查询核对。原始编号也可用 ins_N 调用，如 ins_13(value)。

## 按适用版本查询

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH18、TH19、TH20 | 0 | `end()` | 结束结局／staff 解释器 | 无 |
| TH16、TH18 | 3 | `textAdd(text)` | 写下一行文字；每页五行，第六条先清屏 | string text：结局文字，不使用 TH20 的 END 注音前缀 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19 | 3 | `textAdd(text)` | 写下一行结局文字，分页行数未逐作整理 | string text：文字，不使用 TH20 的 END 注音兼容写法 |
| TH20 | 3 | `textAdd(text)` | 写下一行结局文字，支持注音前缀 | string text：文字或注音前缀；正文行数未完整整理 |
| TH16、TH18 | 4 | `text_clear()` | 隐藏文字，不重置下一行位置；分页使用 waitClear | 无 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 4 | `text_clear()` | 隐藏文字，具体行位置重置行为未逐作整理 | 无 |
| TH16、TH18 | 5 | `wait(maxFrames)` | 等待输入或超时，保留行位置，阅读时冻结脚本时钟 | int maxFrames：最长帧数，负值按 999 帧处理 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 5 | `wait(maxFrames)` | 等待输入或超时 | int maxFrames：最长等待帧数；负值和脚本时钟行为未逐作整理，优先使用正数 |
| TH16、TH18 | 6 | `waitClear(maxFrames)` | 等待后重置行位置，开始新页；阅读时冻结脚本时钟 | int maxFrames：最长帧数；负值按其游戏记录转为 999 帧 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 6 | `waitClear(maxFrames)` | 等待并清理翻页状态，逐作细节未整理 | int maxFrames：最长等待帧数，优先使用正数 |
| TH18 | 7 | `anmSourceLoad(source, file)` | 加载 ANM 到源槽，加载期间暂停 END 脚本 | int source：ANM 源编号；string file：ANM 文件名 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH19、TH20 | 7 | `anmSourceLoad(source, file)` | 加载 ANM 到源槽，加载期间调度细节未逐作整理 | int source：ANM 源编号；string file：ANM 文件名 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH18、TH19、TH20 | 8 | `anmSetSlot(slot, source, script)` | 在显示槽创建或替换 ANM VM | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH16、TH18 | 9 | `textColor(color)` | 设置后续文字颜色，使用 COLORREF | int color：白色 0x00ffffff，红色 0x000000ff |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19 | 9 | `textColor(color)` | 设置后续文字颜色 | int color：颜色参数；具体渲染细节未逐作整理 |
| TH20 | 9 | `textColor(color1, color2)` | 设置后续文字的两个颜色参数 | int color1、color2：各自渲染用途尚未整理，保留已有文件的配对关系 |
| TH18 | 10 | `music(name)` | 播放音乐并解锁音乐室曲目 | string name：如 bgm/th18_15，不带 .wav |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH19、TH20 | 10 | `music(name)` | 播放音乐，音乐室解锁行为未逐作整理 | string name：音乐名／路径，不带 .wav |
| TH18 | 11 | `musicFade()` | 固定 3 秒淡出音乐 | 无 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH19、TH20 | 11 | `musicFade()` | 淡出音乐，时长未逐作整理 | 无 |
| TH16、TH18 | 12 | `switchToStaff(file)` | 按难度选 staff1–staff4.msg；从结局回想进入则直接结束 | string file：该作忽略传入文件名，不能用它实现任意跳转 |
| TH20 | 12 | `switchToStaff(file)` | 按传入文件名进入 staff | string file：staff 文件名 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19 | 12 | `switchToStaff(file)` | 转入 staff，文件选择规则未逐作整理 | string file：当前签名要求字符串，是否使用其内容未逐作确认 |
| TH18 | 13 | `__screen_effect_A(frames)` | 黑幕淡去，画面由黑变亮 | int frames：效果帧数；需用时间标签留出演出时间 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH19、TH20 | 13 | `__screen_effect_A(value)` | 画面效果，方向和具体行为未逐作整理 | int value：当前签名接受整数；不套用 TH18 解释 |
| TH18 | 14 | `screenEffectB(frames)` | 黑幕加深，画面变黑 | int frames：效果帧数；需用时间标签留出演出时间 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH16、TH17、TH19、TH20 | 14 | `screenEffectB(value)` | 画面效果，方向和具体行为未逐作整理 | int value：当前签名接受整数；不套用 TH18 解释 |
| TH16、TH18 | 15 | `anmSetSlotNormal(slot, source, script)` | 仅 Normal 难度设置显示槽 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 15 | `anmSetSlotNormal(slot, source, script)` | 与 Normal 难度相关的显示槽操作，条件分支未逐作整理 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH16、TH18 | 16 | `anmSetSlotHard(slot, source, script)` | 仅 Hard 难度设置显示槽 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 16 | `anmSetSlotHard(slot, source, script)` | 与 Hard 难度相关的显示槽操作，条件分支未逐作整理 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH16、TH18 | 17 | `anmSetSlotLunatic(slot, source, script)` | 仅 Lunatic 难度设置显示槽 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH15、TH17、TH19、TH20 | 17 | `anmSetSlotLunatic(slot, source, script)` | 与 Lunatic 难度相关的显示槽操作，条件分支未逐作整理 | int slot：显示槽；int source：已加载源编号；int script：源 ANM 内脚本号 |

未列编号没有本手册采用的默认定义，不根据编号空缺补造指令。

## 已限定版本的配合说明

- TH16／TH18：等待阅读时脚本时钟冻结，后续时间标签不用计入阅读等待时长。
- TH18：首次观看结局禁止等待快进，看过后可快进翻页。staff 的字幕在 ANM 中，END 调度图片和音乐，不同难度的末张 CG 脚本可以不同。
- TH18：`screenEffectB(60);` 后以 `+60:` 再 end，可避免立刻结束截断淡出。
- TH20：`textAdd("|0,11,あす"); textAdd("明日も晴れるだろう。");` 的前缀依次是横向偏移、字距和注音，本身不占正文行。

所有表列版本的 ANM 源槽保存文件，显示槽保存 VM，二者不能混用；同一源可供多个显示槽使用。默认 map 提供 AnmIndex0 等源编号、AnmSlot0 等显示槽编号，实际脚本号取自目标 ANM。
