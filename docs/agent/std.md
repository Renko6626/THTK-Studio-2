# 编写 STD 背景

STD 负责背景几何实例与相机演出，贴图、平面的绘制模式和局部动画来自 ANM。先选对应世代的完整例子：[TH06](examples/th06-background.stdspec)、[TH08](examples/th08-background.stdspec)、[TH18](examples/th18-background.stdspec)。

## meta：对象、quads 与实例

TH095 起的文件头使用 ANM 路径：

```c
meta {
    unknown: 0,
    anm_path: "stage01.anm",
    objects: {
        tile: {
            layer: 0,
            pos: [-64.0, -64.0, 0.0],
            size: [128.0, 128.0, 0.0],
            quads: [
                rect {anm_script: 0, pos: [0.0, 0.0, 0.0], size: [128.0, 128.0]},
            ],
        },
    },
    instances: [
        tile {pos: [0.0, 256.0, 0.0]},
        tile {pos: [0.0, 384.0, 0.0]},
    ],
}
script main { /* 相机和演出 */ }
```

`objects` 定义可复用模型；一个模型的 `quads` 可组合多个 ANM 脚本，比如地板、两侧墙和屋顶。`instances` 将模型放到关卡空间，同一个对象可以反复出现，实例顺序也会影响二进制输出。

| 字段 | 用途 |
| --- | --- |
| `unknown` | 文件头保留字段，已有文件沿用原值，新例子用 0 |
| 对象 `layer` | 绘制层信息；旧 truth 输出也可能叫 `unknown`，新文件用 `layer` |
| 对象 `pos`、`size` | 包围区域，供游戏剔除判断；没有自动替所有 quad 做平移缩放 |
| quad `anm_script` | 背景 ANM 内的脚本编号；资源必须存在 |
| quad `pos` | 相对该对象实例的局部位置 |
| quad `size` | 平面宽高，可改变贴图显示尺寸 |
| 实例 `pos` | 对象在关卡空间中的放置位置 |

对象包围区域要覆盖实际 quads，否则相机移动时可能提前消失。只扩大对象 `size` 不会把可见平面放大；可见尺寸要改 quad 或 ANM。ANM 负责平面、billboard、旋转等绘制细节，STD 不包含图片数据。

TH06–09 的 meta 把 `anm_path` 换成 `stage_name` 与恰好四项 `bgm: [{path, name}, ...]`，完整格式见 TH06／08 例子。这个文件头世代与脚本 opcode 世代要分别选择。TH08／09 还支持 `strip {anm_script, start, end, width}`，其他 profile 不使用 strip。

## 相机：位置、朝向、up 与 FOV

现代相机初始化（TH095 起）：

```c
    pos(0.0, 0.0, -100.0);     // op2，相机位置
    facing(0.0, 1.0, 0.0);     // op4，朝向向量
    up(0.0, 0.0, -1.0);        // op6，相机的上方向
    fov(0.68831855);           // op7，垂直视场角，弧度
```

`facing(x,y,z)` 是相对相机的朝向向量，目标世界坐标应减去相机位置。`up` 决定画面的上方向；它与 `facing` 一起确定相机姿态。二者都不要求单位长度，但不要使用零向量或相互平行的向量。`up` 在默认 map 的长名为 `camera_rotation`，旧教程有时叫 movement／forward vector，实际编写按上方向理解。

沿用原关卡的坐标轴约定最方便；常见背景以 X 表示左右、Y 表示行进方向，示例用负 Z 放高相机。FOV 改大时可见范围扩大，同一对象在屏幕上显得更小。

## 雾、颜色与插值

```c
    fog(0xff808080, 200.0, 1000.0);                  // op8
    fogTime(60, Linear, 0xff404060, 100.0, 800.0);  // op9
    posTime(120, Linear, 0.0, 128.0, -100.0);       // op3
    facingTime(120, Linear, 0.0, 1.0, 0.2);        // op5
```

TH10 起，常用插值写法为 `持续帧数, 模式, 目标值...`。雾的后两参是距相机的起止距离，near 处开始，far 处达到完全遮挡；安排远处实例消失时让雾先遮住它们。颜色写 `0xAARRGGBB`。`bgColor(color)`（op13）控制清屏色，雾色由 `fog` 单独设置。

串接两段相机移动时，在新段开始先 `pos(...)` 写该段起点，再 `posTime(...)` 写终点，可避免沿用上一次设置的起点。`Linear` 为线性；其他模式枚举随版本变化，例如 TH08 的 `DecelerateSlow=1`，现代的同名模式为 4。保留枚举名比复制数字清楚。

`upTime(t, mode, x,y,z)`（op18）从 TH12 起可用；`fovTime(t, mode, fov)`（op21）从 TH17 起可用。`posBezier`／`facingBezier`（op10／11）各接持续帧、模式和三组三维数值，涉及导数的确切解释尚未完整整理；普通转场可用上面的线性／缓动调用。

## 循环与 ECL 中断

```c
repeat:
    pos(0.0, 0.0, -100.0);
    posTime(120, Linear, 0.0, 128.0, -100.0);
+120:
    goto repeat;
interrupt[1]:
    fogTime(60, Linear, 0xff404060, 100.0, 800.0);
    stop();
```

`goto` 把脚本位置和时间一起恢复到 `repeat`，插值也会每 120 帧重新开始。相机回到开头时若要无缝衔接，需要在空间上重复模型，使两端看到相同图案；符号跳转只处理控制流，不会自动复制场景。`loop { ... +120: }` 也能表示循环，见 [共用语法](truth-syntax.md)。

`interrupt[1]:` 由有效 map 编译为中断标记（TH07–09 为 op31，现代为 op16）。ECL 触发同号 STD 中断时，STD 跳到这个分支；现代作常用 ECL `callSTD`（如 TH13–18 的 opcode 630）。中断可以在 STD 等待时间或 `stop()` 时触发。中断分支末尾用 `stop()` 停住，或 `goto` 返回指定演出段；中断会转移执行位置，结束后不会自动返回原处。TH095／TH10 默认 map 没有中断标记；不要因现代编号接近就加上该功能。

## TH06 与早期关键帧

TH06 的相机位置由 `posKeyframe(x,y,z)`（op0）及其时间标签描述：

```c
    fog(0xff808080, 200.0, 1000.0); // op1
    facing(0.0, 1.0, 0.0);         // op2
    posKeyframe(0.0, 0.0, -100.0);
+120:
    posKeyframe(0.0, 128.0, -100.0);
-1:
    posKeyframe(0.0, 128.0, -100.0);
```

引擎从关键帧序列插值位置；最后一个关键帧使用 `-1:` 哨兵，不能删除。TH06 没有跳转；`pause()`（op5）暂停脚本，由 ECL 对应指令恢复。`facingTime(t)`（op3）与同帧的 `facing` 配合插值朝向。雾变化用 `fogTime(t)`（op4）放在目标 `fog(...)` 前面。

TH07–09 保留关键帧，也增加显式位置、up、FOV、跳转和中断。它们的插值把计时和目标拆开：

```c
    pos(0.0, 0.0, -100.0);  // op5，起点
    posTime(120, Linear);    // op6，只给时长与模式
    pos(0.0, 128.0, -100.0); // 同帧最终目标
```

这一世代 `facing`／`facingTime` 为 op7／8，`up`／`upTime` 为 op9／10，`fov`／`fovTime` 为 op11／12。`fogTime(t)` 为 op2，要放在目标 `fog(...)` 前。固定长度指令的零填充由编译器补齐，常规调用不用手写尾部零。

TH095 的现代 opcode 采用单独 profile，插值参数包含两个模式字节，例如 `posTime(120, Linear, Default, 0.0, 128.0, -100.0)`。这里 `Linear` 是缓动模式，`Default` 是插值类型；两个参数都要写。完整版本分界见 [版本差异](version-differences.md)。

## 其他常用背景操作

| 调用 | opcode | 版本与用途 |
| --- | --- | --- |
| `sprite(slot, script)` | 14 | TH095–13，创建背景 ANM 的二维精灵 |
| `sprite(slot, script, layer)` | 14 | TH14 起增加绘制层，常用于天空和云 |
| `rockingMode(mode)` | 12 | 现代背景摇动，0 关闭，模式效果随游戏变化 |
| `distortion(mode)` | 17 | TH11 起，边缘扭曲；具体位置与模式依游戏变化 |
| `ins_19(n)` | 19 | TH14 起，给全部背景精灵发 ANM 中断 `n+7` |
| `ins_20(distanceSquared)` | 20 | TH14 起，设置距离剔除阈值，单位为距离平方 |

完整参数表来自版本 map。未整理用途的指令集中在 [来源附录](sources.md#尚未整理的行为)，保留原样即可继续修改已经解释清楚的相机和几何部分。
