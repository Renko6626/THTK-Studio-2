# MSG 编程介绍

MSG 控制关卡对话：台词、立绘、气泡、音乐以及与 ECL 的配合。源文件为 UTF-8 的 `.msgspec`，编译产物为 `.msg`。编号、作用和参数见 [MSG 指令参考](msg-instructions.md)；检查和编译命令见 [入口](README.md#版本编码与检查)。

## 文件结构与语法

一个 `meta` 的 `table` 将游戏使用的入口号映射到具名 `script`。入口号由调用方 ECL 决定；多个入口可以引用同一脚本。脚本声明为 `script name { ... }`，没有 ANM 式数字编号。修改已有文件时保留入口 flags、table_len、TH19 header 和资源引用。

字段用冒号连接，字段之间用逗号；结构用 `{}`，数组用 `[]`，字符串用双引号。指令调用以分号结尾。`//` 为行注释；整数 `1` 与浮点数 `1.0` 类型不同。可写 `const int READ_TIME = 600;` 等编译期常量和算术表达式；没有 ECL 的运行时变量、寄存器、条件分支和自定义函数调用。

`40:` 表示绝对帧 40，`+40:` 相对上一时间增加 40；同一标签下的指令使用同一脚本时间。阅读等待时钟冻结的版本不必把等待时长再加到下一标签。普通对话使用非负时间，正常播放一般按 60 帧每秒安排演出。

## 完整小例子：TH18

```c
// TH18: face number comes before portrait slot.
meta { table: {0: {script: "greeting", flags: 256}} }
script greeting {
    skippable(true);
    playerShow(Portrait0);
    speakerPlayer(0);
    playerFace(0, Portrait0);
    bubbleType(TextType0);
    bubblePos(112.0, 240.0);
    textAdd("The rain has stopped.");
    textAdd("Let us take the mountain path.");
+4:
    textPause(600);
    eclResume();
+30:
    textboxHide();
    playerHide(0);
    end();
}
```

例子中的立绘槽和表情取自目标 ANM。TH18 的 `playerFace(表情, 槽号)` 先写表情；`playerShow` 选择本侧槽，不能指定任意 ANM 文件。立绘与气泡外观由关卡资源配合，编译通过不保证资源存在或文字排版正确。

TH16／18 每页两行，第三条 `textAdd` 会清掉上一页；`textPause` 放行后，下一条文本从新页开始。若让 boss 在对话中途登场，在 MSG 对应时间调用 `eclResume()`，让 ECL 的移动接在相应 `dialogWait` 后。结束对话用 `end()`；关卡结算用 `stageEnd()`，要保留原有关卡配合。

## 其他版本怎么写

- TH10 使用相近的现代调用，但多数立绘操作无槽参数；`textAdd` 是 op16、`textPause` 是 op10。TH11 插入旁白后，一组 opcode 后移，原始编号不能直接搬用。
- TH06／07 的典型调用为 `show(LeftPortrait, 0)`、`face(LeftPortrait, 0)`、`anmInterrupt(LeftPortrait, 1)`、`textSet(PaleBlue, TopLine, "Hello.")`、`textPause(600)`。颜色、行号、槽和 ANM 脚本是不同参数。
- TH08／09 有多立绘与 `focusSet`，但组合参数和路线指令有差异。TH19 有额外文件头和相对侧立绘；TH20 也不能直接套用 TH19 的额外编号。完整差异已列在 [指令参考](msg-instructions.md)，无需先阅读一份通用版本文档。

用默认 map 的名称或 `ins_17(...)` 一类十进制 opcode 调用。项目可通过 `#pragma mapfile "../maps/dialogue.msgm"` 覆盖定义；精确签名由实际 map 决定。反编译出的未知 `@blob` 原样保留，正常台词交给编译器处理编码、加密和填充。

项目默认版本与编码仅供参考。按具体文件、目标游戏或补丁及当前任务选择；原版文本通常用 Shift-JIS，汉化与其他修改版按其解码和字库能力选择 GBK／UTF-8。源码始终 UTF-8。
