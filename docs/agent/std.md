# STD 编程介绍

STD 保存背景模型实例和相机演出。ANM 提供贴图、平面的绘制模式与局部动画；STD 自身不包含图片数据。源文件为 UTF-8 的 `.stdspec`，产物为 `.std`。编号、作用、参数和世代差异见 [STD 指令参考](std-instructions.md)。

## 文件结构与语法

`meta.objects` 定义模型，`meta.instances` 把模型放到关卡空间，唯一的 `script main` 编排相机、雾和控制流。TH095 起文件头使用 `anm_path`。字段用冒号、逗号分隔，结构用 `{}`、数组用 `[]`，`rect {...}` 和实例的模型名表示具名结构。

指令以分号结尾；整数 `1` 与浮点数 `1.0` 类型不同。支持编译期常量和表达式，如 `const int MOVE_TIME = 120;`。`+120:` 相对上一时间增加 120 帧，`120:` 设置绝对时间；`repeat:` 是普通标签。TH07–09 和现代 STD 可用 `goto`、`loop`；跳转位置和时间由 truth 编码。没有 ECL 的运行时变量、条件判断或自定义函数调用。

## 完整小例子：TH18

```c
// ANM script 0 must exist in the project background ANM.
meta {
    unknown: 0,
    anm_path: "stage01.anm",
    objects: {
        tile: {
            layer: 0,
            pos: [-64.0, -64.0, 0.0],
            size: [128.0, 128.0, 0.0],
            quads: [rect {anm_script: 0, pos: [0.0, 0.0, 0.0], size: [128.0, 128.0]}],
        },
    },
    instances: [
        tile {pos: [0.0, 256.0, 0.0]},
        tile {pos: [0.0, 384.0, 0.0]},
    ],
}
script main {
    up(0.0, 0.0, -1.0);
    facing(0.0, 1.0, 0.0);
    fov(0.68831855);
    fog(0xff808080, 200.0, 1000.0);
repeat:
    pos(0.0, 0.0, -100.0);
    posTime(120, Linear, 0.0, 128.0, -100.0);
+120:
    goto repeat;
interrupt[1]:
    fogTime(60, Linear, 0xff404060, 100.0, 800.0);
    stop();
}
```

| 字段 | 含义 |
| --- | --- |
| `unknown` | 文件头保留值，已有文件沿用，新例子用 0 |
| 对象 `layer` | 绘制层信息；旧 truth 输出也可能叫 `unknown` |
| 对象 `pos`、`size` | 供剔除判断的包围区域，须覆盖实际 quads；不直接平移或缩放所有 quad |
| quad 的 `anm_script` | 背景 ANM 内的脚本号，资源须存在 |
| quad 的 `pos`、`size` | 相对实例的局部位置、平面宽高；可见尺寸在此或 ANM 中调整 |
| 实例 `pos` | 模型在关卡空间的位置；同一对象可反复实例化 |

`facing` 是方向向量，不是目标点；`up` 决定画面上方向，二者避免零向量或相互平行。例子沿 Y 前进、相机置于负 Z，修改已有背景时沿用原关卡的坐标约定。雾色和清屏色为 `0xAARRGGBB`，不使用旧 thstd 的 `#...` 写法。

`goto repeat` 同时恢复脚本位置与时间，循环不会自动复制几何模型。无缝效果需要安排空间上重复的实例。`interrupt[1]:` 由 ECL 同号 STD 中断触发，转移执行位置后不自动返回；可在分支末尾 `stop()` 或跳回指定标签。

## 早期版本

TH06–09 把 `anm_path` 换成 `stage_name` 与恰好四项 `bgm: [{path, name}, ...]`。TH08／09 还支持 `strip {anm_script, start, end, width}`；其他支持 profile 不使用 strip。文件头世代和脚本 opcode 世代分别选择。

TH06 使用带时间标签的 `posKeyframe(x,y,z)`，末尾保留 `-1:` 哨兵关键帧，没有跳转指令。TH07–09 的位置插值写为 `pos(起点); posTime(120, Linear); pos(终点);`，计时与目标分开。TH095 的 `posTime` 需要 `帧数, 模式, 类型, x,y,z`；TH10 起改为 `帧数, 模式, x,y,z`。TH095／10 没有中断标记。完整调用见 [指令参考](std-instructions.md)。

项目默认版本仅供参考，按具体文件、目标游戏和当前任务选相应世代；STD 不使用 MSG 的二进制文本编码选项。检查和编译命令见 [入口](README.md#版本编码与检查)。语法通过后，资源存在性、模型剔除、画面与时序仍需在目标游戏确认。
