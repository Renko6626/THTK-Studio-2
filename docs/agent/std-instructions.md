# STD 指令参考

人工维护的背景脚本指令说明，文件结构和例子见 [STD 编程介绍](std.md)。支持 TH06、TH07、TH08、TH09、TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20；TH06NC 不在范围内。TH095／TH103 等是独立 profile，不能按整数大小判断继承关系。

先确定当前任务的目标版本，再选择该版本明确列出的行。每行独立，不按某一作基准表套用差异。编号、参数数量或运行行为不同就分行；行为未完整整理的条目保留其已知签名，不借用其他版本的解释。项目默认版本仅供参考。

int、float 是源码类型；颜色为整数 0xAARRGGBB，时间参数以帧计，向量按 x、y、z 顺序。map 的填充由编译器补齐，不作为源码参数。名称采用默认 map，项目自定义签名用现有查询核对；类型和参数数目一致不代表运行语义一致。

## TH095 及之后：逐条适用版本

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH095、TH10 | 0 | `stop()` | 停止后续指令调度，插值继续更新 | 无；这两个版本没有 STD 中断标记指令 |
| TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 0 | `stop()` | 停止后续指令调度，插值继续更新；仍可触发 STD 中断 | 无 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 1 | `jmp(offset, time)` | 跳转并设置脚本时间 | offset：目标字节偏移；time：跳转后时间。推荐用 goto label，由 truth 编码位置和时间 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 2 | `pos(x, y, z)` | 设置相机位置 | float x、y、z：世界坐标 |
| TH095 | 3 | `posTime(frames, mode, type, x, y, z)` | 插值相机位置 | int frames：持续帧数；int mode：TH095 缓动模式；int type：Default=0 或 Bezier=8；float x、y、z：目标位置 |
| TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 3 | `posTime(frames, mode, x, y, z)` | 插值相机位置 | int frames：持续帧数；int mode：TH10 起的插值模式；float x、y、z：目标位置 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 4 | `facing(x, y, z)` | 设置相机朝向向量 | float x、y、z：相对方向，不是目标世界坐标，不要求单位长度 |
| TH095 | 5 | `facingTime(frames, mode, type, x, y, z)` | 插值相机朝向 | int frames：帧数；int mode：TH095 缓动模式；int type：Default=0 或 Bezier=8；float x、y、z：目标方向 |
| TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 5 | `facingTime(frames, mode, x, y, z)` | 插值相机朝向 | int frames：帧数；int mode：TH10 起的插值模式；float x、y、z：目标方向 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 6 | `up(x, y, z)` | 设置相机上方向 | float x、y、z：up 向量，不要求单位长度；长名 camera_rotation |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 7 | `fov(angle)` | 设置垂直视场角 | float angle：弧度 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 8 | `fog(color, near, far)` | 设置距离雾 | int color：0xAARRGGBB；float near：开始距离；float far：完全遮挡距离 |
| TH095 | 9 | `fogTime(frames, mode, type, color, near, far)` | 插值雾参数 | int frames：帧数；int mode：TH095 缓动模式；int type：Default=0 或 None=1；int color：目标 ARGB；float near、far：目标起止距离 |
| TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 9 | `fogTime(frames, mode, color, near, far)` | 插值雾参数 | int frames：帧数；int mode：TH10 起的插值模式；int color：目标 ARGB；float near、far：目标起止距离 |
| TH10、TH103、TH11 | 10 | `posBezier(frames, ax, ay, az, bx, by, bz, cx, cy, cz)` | 位置 Bezier 插值 | int frames：持续帧数；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理；模式字段由编译器填充，源码不传 mode |
| TH095 | 10 | `posBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 位置 Bezier 插值 | int frames：帧数；int mode：map 标注 TH095 InterpMode，完整模式行为未整理；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH12、TH18、TH185、TH19、TH20 | 10 | `posBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 位置 Bezier 插值 | int frames：帧数；int mode：默认 map 标注 Bezier=8；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17 | 10 | `posBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 位置 Bezier 插值 | int frames：帧数；int mode：模式整数，当前 map 未标注枚举，具体可用模式未逐作整理；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH10、TH103、TH11 | 11 | `facingBezier(frames, ax, ay, az, bx, by, bz, cx, cy, cz)` | 朝向 Bezier 插值 | int frames：持续帧数；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理；模式字段由编译器填充，源码不传 mode |
| TH095 | 11 | `facingBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 朝向 Bezier 插值 | int frames：帧数；int mode：map 标注 TH095 InterpMode，完整模式行为未整理；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH12、TH18、TH185、TH19、TH20 | 11 | `facingBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 朝向 Bezier 插值 | int frames：帧数；int mode：默认 map 标注 Bezier=8；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17 | 11 | `facingBezier(frames, mode, ax, ay, az, bx, by, bz, cx, cy, cz)` | 朝向 Bezier 插值 | int frames：帧数；int mode：模式整数，当前 map 未标注枚举，具体可用模式未逐作整理；float ax、ay、az、bx、by、bz、cx、cy、cz：三组三维数值，完整运行语义未整理 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 12 | `rockingMode(mode)` | 设置背景摇动模式并重置计时 | int mode：0 关闭；其他模式按各作，不互相套用效果 |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 13 | `bgColor(color)` | 设置清屏色，与雾色独立 | int color：0xAARRGGBB |
| TH095、TH10、TH103、TH11、TH12、TH125、TH128、TH13 | 14 | `sprite(slot, script)` | 创建背景 ANM 二维精灵 | int slot：背景槽；int script：ANM 脚本号；不传 layer |
| TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 14 | `sprite(slot, script, layer)` | 创建背景 ANM 二维精灵并指定绘制层 | int slot：背景槽；int script：ANM 脚本号；int layer：绘制层，低层先画 |
| TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 16 | `interruptLabel(id)` | 定义中断入口 | int id：中断编号，通常写 interrupt[id]: |
| TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 17 | `distortion(mode)` | 设置画面边缘扭曲 | int mode：模式；效果位置与具体外观依游戏，未逐作整理 |
| TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 18 | `upTime(frames, mode, x, y, z)` | 插值相机上方向 | int frames：帧数；int mode：TH10 起的插值模式；float x、y、z：目标 up 向量 |
| TH14、TH143、TH15、TH16、TH165、TH17 | 19 | `ins_19(n)` | 向全部背景精灵发 ANM 中断 | int n：发送的中断号为 n+7；默认 map 无短名 |
| TH18、TH185、TH19、TH20 | 19 | `__anm_interrupt_all(n)` | 向全部背景精灵发 ANM 中断 | int n：发送的中断号为 n+7 |
| TH14、TH143、TH15、TH16、TH165、TH17 | 20 | `ins_20(distanceSquared)` | 设置距离剔除阈值 | float distanceSquared：距离平方；默认 map 无短名 |
| TH18、TH185、TH19、TH20 | 20 | `__draw_distance(distanceSquared)` | 设置距离剔除阈值 | float distanceSquared：距离平方 |
| TH17、TH18、TH185、TH19、TH20 | 21 | `fovTime(frames, mode, angle)` | 插值垂直视场角 | int frames：帧数；int mode：TH10 起的插值模式；float angle：目标弧度 |
| TH20 | 22 | `__unknown_flag_A(value)` | 行为未整理 | int value：map 标注布尔值，具体功能未整理 |

### 按版本查插值模式

下面的值用于对应版本的 mode 参数，不能跨行套用。TH095 的 type 参数与 mode 是两个独立参数，允许的 type 已直接写在对应指令行中。

| 适用版本 | 已整理的 mode 值 | 使用范围 |
| --- | --- | --- |
| TH095 | Linear=0；减速慢／中／快=1／2／3；加速慢／中／快=4／5／6 | op3、op5、op9 的缓动模式；其 Bezier=8 是 type 值，不是本表的 mode 枚举 |
| TH10、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | Linear=0；加速慢／中／快=1／2／3；减速慢／中／快=4／5／6；ConstantVelocity=7；Bezier=8 | 普通插值的常用值；Bezier 指令单独按各自条目，不能把所有模式用于所有插值指令 |

部分版本的默认 map 未提供枚举别名，此时使用已确认的整数。其他模式值以对应 map 和已知游戏资料为准，不从相邻作品推断。

## TH06：逐条适用版本

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH06 | 0 | `posKeyframe(x, y, z)` | 相机位置关键帧 | float x、y、z：位置，取指令时间标签；末尾需 -1: 哨兵关键帧 |
| TH06 | 1 | `fog(color, near, far)` | 设置距离雾 | int color：ARGB；float near、far：开始与完全遮挡距离 |
| TH06 | 2 | `facing(x, y, z)` | 设置朝向 | float x、y、z：方向向量 |
| TH06 | 3 | `facingTime(frames)` | 设置朝向插值计时 | int frames：帧数，与同帧 facing 配对 |
| TH06 | 4 | `fogTime(frames)` | 设置雾插值计时 | int frames：帧数，放在目标 fog 前 |
| TH06 | 5 | `pause()` | 暂停脚本，由 ECL 对应操作恢复 | 无有效参数 |

TH06 没有跳转指令，不能使用其他版本的 goto 或 stop。引擎从关键帧序列插值位置，末尾负时间哨兵需要保留。

## TH07–TH09：逐条适用版本

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH07、TH08、TH09 | 0 | `posKeyframe(x, y, z)` | 位置关键帧 | float x、y、z：位置 |
| TH07、TH08、TH09 | 1 | `fog(color, near, far)` | 设置距离雾 | int color：ARGB；float near、far：开始与完全遮挡距离 |
| TH07、TH08、TH09 | 2 | `fogTime(frames)` | 设置雾插值计时 | int frames：帧数，放在目标 fog 前 |
| TH07、TH08、TH09 | 3 | `stop()` | 停止指令调度 | 无有效参数 |
| TH07、TH08、TH09 | 4 | `jmp(offset, time)` | 跳转并恢复时间 | offset：按指令序号编码的目标；time：脚本时间。推荐用 goto label 由编译器处理 |
| TH07、TH08、TH09 | 5 | `pos(x, y, z)` | 设置相机位置／插值目标 | float x、y、z：坐标 |
| TH07、TH08、TH09 | 6 | `posTime(frames, mode)` | 设置位置插值计时与模式 | int frames：帧数；int mode：TH07–TH09 模式；目标由同帧 pos 提供 |
| TH07、TH08、TH09 | 7 | `facing(x, y, z)` | 设置朝向／插值目标 | float x、y、z：方向 |
| TH07、TH08、TH09 | 8 | `facingTime(frames, mode)` | 设置朝向插值计时与模式 | int frames：帧数；int mode：TH07–TH09 模式；目标由同帧 facing 提供 |
| TH07、TH08、TH09 | 9 | `up(x, y, z)` | 设置相机上方向／插值目标 | float x、y、z：up 向量 |
| TH07、TH08、TH09 | 10 | `upTime(frames, mode)` | 设置上方向插值计时与模式 | int frames：帧数；int mode：TH07–TH09 模式；目标由同帧 up 提供 |
| TH07、TH08、TH09 | 11 | `fov(angle)` | 设置视场角／插值目标 | float angle：弧度 |
| TH07 | 12 | `fovTime(frames, mode)` | 设置视场角插值计时 | int frames：帧数；int mode：TH07 的 InterpMode；目标由同帧 fov 提供 |
| TH08、TH09 | 12 | `fovTime(frames, mode)` | 设置视场角插值计时 | int frames：帧数；int mode：本作受限 InterpModeLimited；目标由同帧 fov 提供 |
| TH07、TH08、TH09 | 13 | `clearColor(color)` | 设置清屏色 | int color：ARGB |
| TH07、TH08、TH09 | 14 | `posInitial(x, y, z)` | 位置曲线的起始向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 15 | `posFinal(x, y, z)` | 位置曲线的结束向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 16 | `posInitialDeriv(x, y, z)` | 位置曲线的起始导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 17 | `posFinalDeriv(x, y, z)` | 位置曲线的结束导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 18 | `posBezier(frames)` | 位置 Bezier 插值计时 | int frames：持续帧数；与 op14–17 配合的完整行为未整理 |
| TH07、TH08、TH09 | 19 | `facingInitial(x, y, z)` | 朝向曲线的起始向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 20 | `facingFinal(x, y, z)` | 朝向曲线的结束向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 21 | `facingInitialDeriv(x, y, z)` | 朝向曲线的起始导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 22 | `facingFinalDeriv(x, y, z)` | 朝向曲线的结束导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 23 | `facingBezier(frames)` | 朝向 Bezier 插值计时 | int frames：持续帧数；与 op19–22 配合的完整行为未整理 |
| TH07、TH08、TH09 | 24 | `upInitial(x, y, z)` | 上方向曲线的起始向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 25 | `upFinal(x, y, z)` | 上方向曲线的结束向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 26 | `upInitialDeriv(x, y, z)` | 上方向曲线的起始导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 27 | `upFinalDeriv(x, y, z)` | 上方向曲线的结束导数向量 | float x、y、z：三个分量；曲线配合及导数尺度的完整运行语义未整理 |
| TH07、TH08、TH09 | 28 | `upBezier(frames)` | 上方向 Bezier 插值计时 | int frames：持续帧数；与 op24–27 配合的完整行为未整理 |
| TH07、TH08、TH09 | 29 | `spriteA(script)` | 背景 ANM 脚本相关，具体槽和显示行为未整理 | int script：ANM 脚本号 |
| TH07、TH08、TH09 | 30 | `spriteB(script)` | 背景 ANM 脚本相关，具体槽和显示行为未整理 | int script：ANM 脚本号 |
| TH07、TH08、TH09 | 31 | `interruptLabel(id)` | 定义中断入口 | int id：中断编号，通常写 interrupt[id]: |
| TH08、TH09 | 32 | `rockVector(x, y, z)` | 摇动向量 | float x、y、z：分量，坐标与振幅细节未完整整理 |
| TH08、TH09 | 33 | `rockMode(mode)` | 设置摇动模式 | int mode：0 关闭；其他模式的具体效果依目标游戏 |
| TH08、TH09 | 34 | `spriteC(script)` | 背景 ANM 脚本相关，与 spriteA／B 的区别未整理 | int script：ANM 脚本号 |

这三个版本把插值计时与目标拆开，例如 `pos(起点); posTime(120, Linear); pos(终点);`，计时与终点在同一帧设置。不要套用 TH10 起把终点直接传给 posTime 的写法。

TH07／TH08／TH09 的普通 InterpMode：Linear=0，减速慢／中／快=1／2／3，加速慢／中／快=4／5／6。TH08／TH09 的 op12 使用受限枚举，完整值按对应 map；TH10 起同名减速模式的数字不同。

## 相机与控制流配合

上述已列出的相机接口中，facing 与 up 共同确定姿态，应避免零向量或相互平行；目标点转成 facing 时用目标位置减相机位置。常见的空间重复模型与 goto 配合可形成循环背景，goto 只处理执行位置和时间，不会自动复制几何。

TH07、TH08、TH09、TH103、TH11、TH12、TH125、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 有中断标记接口；分支转移执行位置，不自动返回，末尾可 stop 或跳到其他标签。TH13、TH14、TH15、TH16、TH17、TH18 的 ECL 常用 callSTD（opcode 630）触发同号中断；不据此推断其他作品的 ECL 编号。
