# END 编程介绍

END 编排结局与 staff：文字翻页、CG、音乐和黑幕效果。源文件为 UTF-8 的 `.endspec`，产物也常叫 `.msg`，但指令集独立于关卡 MSG；Studio 选 `ending`，直接用 truth 时加 `--ending`。编号、作用和参数见 [END 指令参考](ending-instructions.md)。

## 文件结构与语法

`meta.table` 将入口编号映射到 `script`，通常只有入口 0。保留原文件的 flags 和入口结构。字段用冒号，字段之间用逗号，指令以分号结尾；字符串双引号，常量可写 `const int READ_TIME = 600;`。整数与浮点数区分，不使用 ECL 的局部变量、寄存器和运行时条件分支。

`+60:` 将脚本时间相对上一标签增加 60 帧，`60:` 设置绝对时间；同一时间下的调用按顺序处理。TH16／18 的阅读等待冻结脚本时钟，后续时间标签不用再计入阅读等待时长。

## 完整小例子：TH18

```c
// Uses existing project resources; no ANM or BGM assets are supplied.
meta { table: {0: {script: "ending", flags: 256}} }
script ending {
    anmSourceLoad(AnmIndex0, "e01.anm");
    anmSetSlot(AnmSlot0, AnmIndex0, 0);
    music("bgm/th18_15");
    textColor(0xffffff);
    textAdd("The journey ends here.");
    textAdd("Tomorrow will bring another visitor.");
+4:
    waitClear(600);
    textAdd("Until then, the mountain rests.");
+4:
    waitClear(600);
    musicFade();
    screenEffectB(60);
+60:
    end();
}
```

ANM、BGM 路径和脚本号必须换成项目实际资源。`anmSourceLoad` 的源编号保存文件，`anmSetSlot` 的显示槽保存当前 VM；同一个源可以供多个显示槽使用。

TH16／18 一页五行；第六条文本先清屏。`waitClear` 等待后开始新页，`wait` 保留行位置。`text_clear` 仅隐藏文字，不作为稳定翻页方式。

发出黑幕或音乐效果后，脚本仍继续按时间执行。例子给 `screenEffectB(60)` 留出 60 帧后才结束，避免立即截断演出。

## staff、版本与编码

TH16／18 的 `switchToStaff("staff1.msg")` 不按传入文件名选择，游戏按难度选 staff；TH20 才使用文件名。TH18 staff 的字幕在 ANM 资源中，END 主要调度图片与音乐，修改图文需同时处理 ANM。

支持 TH10、11、12、128、13、14、15、16、17、18、19、20；TH06–09 旧结局不使用这套接口。TH20 的 `textColor` 需要两个整数颜色，`textAdd` 可使用注音前缀；其余具体参数与版本限制见 [指令参考](ending-instructions.md)。

源码始终 UTF-8。项目默认游戏版本与二进制编码仅供参考，目标版本和 Shift-JIS／GBK／UTF-8 按文件、游戏补丁与当前任务选择。编译不赋予游戏额外解码或字库能力。检查和编译命令见 [入口](README.md#版本编码与检查)。
