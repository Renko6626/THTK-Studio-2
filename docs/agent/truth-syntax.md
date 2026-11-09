# truth 源码写法

## 文件结构

MSG 和 END 各有一个 `meta` 和若干个具名 `script`。入口表把游戏使用的编号映射到脚本名：

```c
const int READ_TIME = 60 * 10;
meta {
    table: {
        0: {script: "beforeBoss", flags: 256},
        1: {script: "afterBoss", flags: 256},
    },
}
script beforeBoss { textAdd("Hello."); textPause(READ_TIME); end(); }
script afterBoss { end(); }
```

上面调用对应 TH11–17 的默认 MSG map；TH18 的完整例子见 [th18-dialogue.msgspec](examples/th18-dialogue.msgspec)。TH06 的文本参数不同，见 [MSG](msg.md)。END 使用同样的入口结构，但采用 [END 指令](ending.md)。脚本声明写 `script name`，不加 ANM 式数字编号。STD 只有一个 `script main`，其 `meta` 定义对象和实例，见 [STD](std.md)。

`meta` 的字段和值用冒号连接，字段以逗号分隔，允许末尾逗号。对象用 `{}`、数组用 `[]`；`rect {...}`、`tile {...}` 是有名字的结构值。字段名不加引号。格式所需字段按对应手册填写。

## 常量、字符串和数值

```c
const int PAGE_WAIT = 60 * 10;
const float CAMERA_HEIGHT = -100.0;
const string GREETING = "Hello, visitor.";
```

常量在编译时求值，可放在顶层或块中。整数支持十进制、`0x` 十六进制和 `0b` 二进制，布尔值为 `true`、`false`；浮点数写 `1.0` 或 `1f`。`1` 和 `1.0` 分别是整数与浮点数，指令参数须匹配类型。可用常量算术、位运算、比较及显式转换 `_S(...)`、`_f(...)`。MSG／END／STD 没有普通运行时变量和算术指令，不能把 ECL 的寄存器、随机数、局部变量计算照搬过来。

字符串使用双引号，支持 `\0`、`\n`、`\r`、`\\`、`\"` 转义。台词通常每行调用一次文本指令，换行和翻页由该格式的文本指令管理。注音前缀直接写 `"|0,11,あす"`，`|` 不需要反斜杠。源文件始终是 UTF-8，MSG／END 二进制编码由工具参数选择。

颜色用整数十六进制。STD 的雾和清屏色使用 `0xAARRGGBB`，例如 `0xff406080`；END 的颜色参数见 [结局文字](ending.md#文字和翻页)，不要套用旧 thstd 的 `#...` 语法。

## 时间标签

```c
script greeting {
    textAdd("First line.");   // 时间 0
+4:
    textPause(600);            // 时间 4，等待时脚本时钟冻结
+30:
    end();                    // 时间 34
}
```

`40:` 设置绝对时间 40，`+40:` 在前一个时间上增加 40；`-40:` 是绝对时间负 40。多个调用之间没有新时间标签时，它们使用同一时间。标签表达的是脚本时钟，`textPause` 的实际阅读等待无需再计入下一条时间标签。MSG 指令时间以无符号 16 位存储，正常对话使用非负时间。早期 STD 末尾的 `-1:` 有专门用途，见 [STD 的关键帧](std.md#th06-与早期关键帧)。

## 标签与控制流

格式对可用控制流的约束如下：

| 格式 | 可直接采用的写法 |
| --- | --- |
| MSG、END | 时间标签和顺序调用；默认指令集没有跳转与条件分支 |
| TH06 STD | 时间标签、位置关键帧与暂停；没有跳转指令 |
| TH07–09、TH095 起 STD | `label:`、`goto label;`、`loop { ... }`，由默认 map 的跳转 intrinsic 编译 |

STD 的循环写法：

```c
repeat:
    pos(0.0, 0.0, -100.0);
    posTime(120, Linear, 0.0, 128.0, -100.0); // TH10 起
+120:
    goto repeat;
```

`goto` 同时恢复目标标签的时间，因此循环在这里每 120 帧重复。`goto label @ 10;` 可以显式指定跳转后的时间；一般把标签放在正确时间就足够。`offsetof(label)`、`timeof(label)` 可用于需要直接写 `jmp(...)` 的场合，编译器会处理各世代的目标编码。日常编辑优先用标签和 `goto`。

通用 truth 语法中的 `if`、`while`、`times` 依赖目标语言的条件跳转和寄存器，不构成 MSG／END／STD 的可用运行时逻辑。`return`、自定义函数和 ECL 式调用也不作为这三类脚本的结束或复用方式。结束 MSG／END 用 `end()`；停止 STD 用对应版本的 `stop()` 或 `pause()`。

## 指令名、原始 opcode 与 map

```c
textAdd("Hello.");
ins_17("Hello."); // TH18 MSG 中与上一条是同一个 opcode
```

`ins_17` 使用十进制 opcode，不能写成 `ins_017`。参数仍按有效签名检查；原始 opcode 写法不会绕开类型检查。未知签名的反编译结果可能保留为：

```c
ins_101(@blob="00000000");
```

`@blob` 是已经编码的原始参数字节，以十六进制对书写，长度须为四字节倍数。`@mask` 表示寄存器参数位掩码，在具有寄存器的语言中使用；MSG／STD 不提供寄存器参数。保留未知指令时原样保留 blob，普通台词用字符串参数，让编译器负责编码、掩码与填充。

源码可以引用相对路径的 map：

```c
#pragma mapfile "../maps/dialogue.msgm"
```

Studio 以源文件目录启动 truth，因此 pragma 的相对路径以源文件目录为基准。直接在终端调用 truth 时，相对路径以进程当前目录为基准；带相对 pragma 的文件先切到其目录再编译。编译器先加载内建签名，再加载命令行 `-m`，再加载源码 pragma。Studio 在命令行 map 阶段依次应用版本默认 map 与项目 map。后加载的定义覆盖前面的定义；改过签名的 opcode 要按项目实际定义编写。MSG／END／STD 分别使用 `.msgm`、`.endm`、`.stdm`；例子通过编译命令加载版本 map，因此不依赖本机绝对路径 pragma。
