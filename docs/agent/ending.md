# 编写结局与 staff END

结局和 staff 使用 `.endspec`；编译时选 `ending` 类型，直接调用 truth 时加 `--ending`。入口表仍使用 `meta.table`，通常只有入口 0。可从 [TH18 结局](examples/th18-ending.endspec) 或 [TH20 结局](examples/th20-ending.endspec) 开始。

END 的 opcode 独立于关卡 MSG：END 的 op3 是 `textAdd`，MSG 的 op3 是旧文本框指令。即使产物都叫 `.msg`，也要选择正确类型和 `.endm`。

## 文字和翻页

TH16／18 的一页有五行：

```c
    textColor(0xffffff);            // op9，后续文本颜色
    textAdd("The journey ends.");   // op3，写下一行
    textAdd("The mountain rests.");
+4:
    waitClear(600);                 // op6，等输入，然后从新页开始
    textAdd("A new day begins.");
+4:
    wait(600);                      // op5，等待，保留行位置
```

第六条 `textAdd` 会先清掉上一页并从第 0 行写起。`waitClear(max)` 适合分页；`wait(max)` 适合同页内的停顿。两者等待时脚本时钟冻结，后续时间标签不用包含阅读时长。TH16／18 的负等待参数按 999 帧处理；跨版本写明确的正数帧数最直观。TH18 首次观看的结局禁止等待快进，看过之后可快进翻页。

`ins_4()`（`text_clear`）只隐藏文字，不重置下一行位置；要稳定开始下一页，用 `waitClear`。TH10–19 的 `textColor(color)` 只有一个整数，TH16／18 按 COLORREF 保存，常用白色 `0x00ffffff`，红色 `0x000000ff`。TH20 必须写两个颜色：`textColor(0xffffff, 0xffffff)`；修改已有文本颜色时保留两个值，两个颜色各自的渲染用途尚未整理。

TH20 的 `textAdd` 可在正文前设置注音：

```c
    textAdd("|0,11,あす");
    textAdd("明日も晴れるだろう。");
```

第一条只给下一行设置注音；前两个值是横向偏移和字距。TH10–19 不使用这套 END 注音写法。

## ANM 源与显示槽

```c
    anmSourceLoad(AnmIndex0, "e01.anm");  // op7，源编号、文件名
    anmSetSlot(AnmSlot0, AnmIndex0, 0);   // op8，显示槽、源编号、ANM 脚本号
```

源槽存放加载的 ANM 文件，显示槽存放正在显示的 ANM VM；可以从同一源创建多个不同显示槽，也可以用同一显示槽替换图片。TH18 加载 ANM 时会暂停 END 脚本，加载完成后继续，无需另加固定加载延迟。例子中的文件与脚本号必须存在于目标项目。

`anmSetSlotNormal(slot, source, script)`、`anmSetSlotHard(...)`、`anmSetSlotLunatic(...)`（op15／16／17）与 op8 参数相同，只在对应难度执行。TH16／18 的实现有这些条件分支；常见游戏资源通过不同结局文件选择 CG。

## 音乐和画面效果

| 调用 | opcode | 写法与效果 |
| --- | --- | --- |
| `music("bgm/th18_15")` | 10 | 路径不带 `.wav`；TH18 同时更新音乐室解锁 |
| `musicFade()` | 11 | TH18 固定淡出 3 秒 |
| `ins_13(60)` | 13 | TH18 黑幕逐渐消失，画面从黑变亮，参数为帧数 |
| `screenEffectB(60)` | 14 | TH18 黑幕逐渐加深，参数为帧数 |
| `switchToStaff("staff1.msg")` | 12 | 进入 staff；文件选择规则见下文 |
| `end()` | 0 | 结束当前结局／staff 解释器 |

黑幕指令发出演出后，后续仍按脚本时间执行。比如 `screenEffectB(60); +60: end();` 给淡出留足 60 帧。不要在同一时间立刻 `end()`，从而截断演出。

TH16／18 的 `switchToStaff` 不使用文件名参数，游戏按难度选 `staff1`–`staff4.msg`；从结局回想菜单进入时会直接结束，继续通关的失败结局通常不接 staff。TH20 开始按传入的文件名选择 staff。修改 TH18 的参数不能用来实现任意 staff 跳转。

TH18 的 staff 复用 END 解释器，主要调度 ANM 图片与音乐，字幕已经画在 ANM 资源里；要改职员表图文需配合 ANM。不同难度 staff 的末张 CG 脚本可以不同。

## 版本与资源

本手册支持 TH10–20 中的 12 个 END profile，详见 [矩阵](support-matrix.md)。TH06–09 的旧结局格式不走这套接口。TH20 的双颜色参数与注音会改变源码写法；其余常用结构见 [版本差异](version-differences.md)。源码保持 UTF-8，二进制使用项目指定的 Shift-JIS／GBK／UTF-8。
