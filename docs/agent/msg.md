# 编写关卡 MSG 对话

从 [TH18 完整例子](examples/th18-dialogue.msgspec) 开始。下面先讲 TH11 起的现代对话，TH10 和 TH06–09 的写法在后半部分。

## 入口表与 ECL

```c
meta {
    table: {
        0: {script: "beforeBoss", flags: 256},
        1: {script: "afterBoss", flags: 256},
    },
}
```

入口号由调用对话的 ECL 决定。TH16／18 的 ECL `dialogRead(0)`（opcode 518）从入口 0 开始；入口表中的脚本名可自由命名，重命名时同步修改 `script`。多个入口可以引用同一脚本。`table_len` 可保留表长度，稀疏表用 `default: {script: "fallback", flags: 256}` 填补空位；没有 default 的空位写为零。改已有文件时保留 `flags`、`table_len` 和 TH19 的 `header` 等元数据。

TH19 的附加头由 `meta.header` 表示，flags 仍是一个 32 位整数。它们参与游戏选择对话的逻辑，移植时保留目标版本反编译所得的结构。编译器处理二进制偏移，不需要手算。

## 写一页台词与等待

TH18：

```c
    playerShow(Portrait0);         // op1，加载左侧立绘
    speakerPlayer(0);              // op7，左侧槽 0 说话
    playerFace(2, Portrait0);      // op13，表情 2，槽 0
    bubbleType(TextType0);         // op29，气泡样式
    bubblePos(112.0, 240.0);       // op28，气泡位置
    textAdd("The rain has stopped."); // op17，第一行
    textAdd("Let us go.");            // 第二行
+4:
    textPause(600);                // op11，等输入，最多 600 帧
```

TH16／18 的对话每页两行。第三次 `textAdd` 会清除上一页，再写第一行；`textPause` 放行后，下一条文本也会从新页开始。要在两页之间给 ANM 演出留时间，用 `+N:`；等待阅读的 600 帧无需加到下一标签。

`skippable(true)`（op10）启用长按快进，`false` 禁用。对话时间标签控制正常播放的演出节奏，快进时游戏会跳过这些等待。TH18 气泡位置参数进入引擎后按两倍坐标使用，调整时沿用原文件的坐标尺度。

TH12 起 `textAdd("|x偏移,字距,注音")` 给下一行设置注音，本身不占文本行，例如：

```c
    textAdd("|0,11,あす");
    textAdd("明日も晴れるだろう。");
```

编译器处理 `furibug` 字符串尾部兼容数据，无需手写垃圾字节。`textLine1`／`textLine2`（op15／16）直接设置指定行；正常分页对话使用 `textAdd`。

## 说话者、立绘和表情

同一侧可以有多个立绘，槽号与表情号是两个概念。TH18 的表情调用顺序是 **表情、槽号**；不要按某些社区表里的参数名把它们倒过来。

| 任务 | TH18 默认 map 调用 | opcode |
| --- | --- | --- |
| 左／右侧立绘登场 | `playerShow(0)` / `bossShow(0)` | 1 / 2 |
| 左／右侧立绘退场 | `playerHide(0)` / `bossHide(0)` | 4 / 5 |
| 指定左／右侧说话者 | `speakerPlayer(0)` / `speakerBoss(0)` | 7 / 8 |
| 旁白，全部变暗 | `speakerNone()` | 9 |
| 左／右侧换表情 | `playerFace(2, 0)` / `bossFace(2, 0)` | 13 / 14 |
| 只改气泡朝向 | `modern32(0)`，0 左、1 右、2 无人 | 32 |
| 单独变暗／高亮 | `portraitDarken(1, 0)` / `portraitHighlight(1, 0)` | 33 / 34 |
| 左／右立绘抖动 | `playerShake()` / `bossShake()` | 23 / 24 |
| 隐藏文字和气泡 | `textboxHide()` | 6 |
| 清掉文字 | `textClear()` | 18 |

`portraitDarken`／`portraitHighlight` 的第一参为侧别（0 左、1 右），第二参为槽。`playerHide` 等虽在 map 中使用 `PortraitSide` 枚举注解，TH18 的实际参数用于本侧槽选择；写多立绘时使用明确整数。`playerShake`／`bossShake` 没有参数，TH18 固定作用于本侧 0、1 号立绘。

立绘实际资源取自自机或关卡的 ANM 配置，MSG 的 `bossShow(1)` 选择关卡配置中的另一敌方立绘，不能指定任意 ANM 文件。TH16／18 的表情通过 ANM 中断 `17 + 表情号` 切换；立绘用 1 隐藏、2 高亮、3 变暗、7 抖动。新画的立绘要在 ANM 中配合这一协议。TH06 的 ANM 中断编号不同，见下文。

`bubbleType`（TH128 起）选择样式，0–3 常见为圆角、尖刺、思考云、说话云；其余外观取决于对应游戏的气泡 ANM。TH16 的 4–7 用于附身红框，不能据此推断每作同号外观。`textOffsetY(dy)`（op25）整体移动文本；TH16／18 的 `textboxShow()`（op3）已不负责现代气泡初始化。

## BGM、名牌、ECL 放行和结束

| 任务 | 现代 MSG 调用 | 版本 |
| --- | --- | --- |
| 播 boss 曲并显示曲名 | `musicBoss()`，op19 | TH11–18、TH20 |
| 选择曲目 | `musicBoss(1)`，op19 | TH185／19 |
| 显示 boss 名牌 | `intro()`，op20；TH14 起 `intro(0)` | TH11 起 |
| 默认时长淡出 | `musicEnd()`，op22 | TH11 起 |
| 指定秒数淡出 | `musicFade(2.0)`，op27 | TH12 起；TH18 已有参数行为记录 |
| 放行 ECL | `eclResume()`，op12 | TH11 起 |
| 结束对话 VM | `end()`，op0 | 全部支持版本 |
| 进入关卡结算流程 | `stageEnd()`，op21 | TH11 起 |

TH16／18 用 ECL `dialogRead` 起对话，ECL 自身继续执行；`dialogWait`（519）等待 MSG 发出 `eclResume()` 或整个对话结束。要让 boss 在对话中途入场，把 `eclResume()` 放在入场台词后的时间点，并让 ECL 的移动逻辑接在对应的 `dialogWait` 后。一条放行是短脉冲，不要把它当成永久开关。

TH18 `musicEnd()` 默认淡出 8 秒，六面为 2 秒；`musicFade(seconds)` 可以指定时长。`store()`（op36）在 TH18 标记对话后开卡牌商店，这条功能不能直接照搬到 TH185 黑市。`lightsOut()`（op35，TH16／17／18 的这组表）用黑幕遮住关卡背景与敌机。

`end()` 释放对话 VM；`stageEnd()` 走关卡流程，二者用途不同。保留已有 ECL 的结算配合，普通 boss 开场对话通常用 `end()` 收尾。

## TH10：用自己的编号和参数

[th10-dialogue.msgspec](examples/th10-dialogue.msgspec) 给出完整写法。主要调用是 `playerShow()`、`speakerPlayer()`、`playerFace(face)`、`textAdd(text)`、`textPause(max)`、`eclResume()`。前两个不传槽号；`playerShow` 的填充参数由编译器补零。

TH10 的 `skippable` 为 op9，`textPause` 为 op10，`eclResume` 为 op11，`textAdd` 为 op16。TH11 在 op9 插入 `speakerNone`，之后一组指令整体后移，不能直接复制原始 opcode。

## TH06–09：显式文本行与旧立绘接口

[th06-dialogue.msgspec](examples/th06-dialogue.msgspec) 的核心写法：

```c
    show(LeftPortrait, 0);                    // op1，立绘槽、ANM 脚本
    face(LeftPortrait, 0);                    // op2，立绘槽、sprite
    anmInterrupt(LeftPortrait, 1);            // op5，让该立绘出现
    textSet(PaleBlue, TopLine, "Hello.");     // op3，颜色、行号、文本
    textSet(PaleBlue, BottomLine, "Welcome.");
    textPause(600);                           // op4
    eclResume();                              // op6
    end();                                   // op0
```

TH06／07 的两行文本使用 op3 指定行，颜色枚举 `PaleBlue=0`、`PaleRed=1`。立绘槽 `LeftPortrait=0`、`RightPortrait=1`；表情用 ANM sprite 号，资源编号来自实际立绘文件。旧立绘常用中断 1 登场、3 高亮、4 变暗、5 退场，不能沿用现代中断 1 隐藏的含义。旧 `intro(color, line, text)`（op8）直接写 boss 名字和称号。

TH08 的四个槽为 `LeftPortraitA=0`、`LeftPortraitB=1`、`RightPortraitA=2`、`RightPortraitB=3`。可以用 `focusSet(who, sprite)`（op17）同时切换说话者和表情，然后由 `textAdd(text)`（op16）写文本：

```c
    show(LeftPortraitA, 0);
    focusSet(LeftPortraitA, 0);
    textAdd("The path is open.");
+4:
    textPause(600);
```

多立绘同时换表情用 `focusSetEx(who, sprite0, sprite1, sprite2, sprite3)`（op15）。六面选路线使用 `optionA(text)`／`optionB(text)`（op19／20）设置候选文字，`optionPause(max)`（op21）设自动选择前的等待；修改候选文字时保留原关卡的分支结构。

TH09 的 `show(who)`（op1）只接一个有效参数，脚本选择逻辑由游戏决定；`focusSet(who,sprite)` 和 `textAdd(text)` 继续可用，槽名为 `Portrait0`–`Portrait3`。组合调用缩成 `focusSetEx(who, sprite0, sprite1)`，其中 `who=-1` 表示无人说话；TH08 的四表情写法不能原样迁移。TH09 删除了 TH08 的 op19–22 路线指令。

TH06／07 明文、TH08 固定 XOR、TH09 起加速 XOR 都由编译器完成。跨版本迁移同时检查 [版本差异](version-differences.md)。

## 编码

源文件 UTF-8。原版二进制选择 `shift-jis`；GBK 汉化或 UTF-8 修改版按目标游戏补丁选择对应编码。切换选项仅改变输出字节，不会给游戏增加解码或字库能力。包含不可编码字符时 truth 会报错，见 [工具使用](tooling.md)。
