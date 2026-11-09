# 按 profile 处理版本差异

游戏号使用工具链整数：TH09.5 为 `95`，TH10.3 为 `103`，TH12.8 为 `128`，TH14.3 为 `143`，TH16.5 为 `165`，TH18.5 为 `185`。按对应的 `.msgm`／`.endm`／`.stdm` 选择写法；整数大小不表示继承顺序。完整集合见 [支持矩阵](support-matrix.md)。

## MSG

| profile | 对编程的影响 |
| --- | --- |
| TH06、07 | 旧 `show(who, script)`、`face(who, sprite)`、`textSet(color,line,text)`；文本明文 |
| TH08 | 文本改固定 XOR，加入多立绘说话者和六面路线文本等；编译器自动加密 |
| TH09 | `show(who)` 的脚本参数变填充；多立绘组合参数由四个表情缩为两个；op8 无参；入口开始保留 flags；采用加速 XOR |
| TH10 | 新立绘接口；`textAdd` 为 op16，`textPause` 为 op10，`eclResume` 为 op11 |
| TH11 | op9 插入 `speakerNone`；TH10 的 op9–23 后移一位；新增 op25／26 |
| TH12 | 新增 `musicFade(seconds)` op27，文本签名加入注音尾部兼容 |
| TH128 | `bossShow` 有有效整数参数；新增气泡位置 op28、气泡类型 op29 与路线选择 op30 |
| TH13 | 新增 op31；当前默认 map 的 `showEnemy1()` 为零参 |
| TH14 | `bossHide(who)`、`speakerBoss(who)`、`bossFace(face,who)`、`intro(who)`，新增 op32 |
| TH143 | op33 是单整数的教程指令；不能写 TH16 的立绘变暗调用 |
| TH15 | 移除 TH143 的 op33 |
| TH16、17 | `playerShow` 开始使用有效参数；op33／34 为两参的立绘变暗／高亮，op35 为黑幕 |
| TH165 | 回到 TH143 分支：op33 单整数，op34／35 不可用 |
| TH18 | `playerHide(who)`、`speakerPlayer(who)`、`playerFace(face,who)`；op36 为商店标志 |
| TH185 | `musicBoss(index)`；新增 op37–41 的说明界面与文字指令 |
| TH19 | 入口前新增 0x50 字节头；保留 `meta.header` 和 flags；删除 op31；op19 带曲目号；op20 成填充；增加 op42–56 |
| TH20 | 当前 map 回到 TH18 的常用调用形状：op19 无参、op20 带参；没有 TH185／19 的 op37–56，也没有 op31 |

TH18 的 `playerShake()`／`bossShake()` 仍无参数；某些旧记录把 TH18 新增参数扩大到了抖动指令，当前 map 不采用那种写法。TH19 的相对侧立绘和附加头缺少 TH16／18 同等的一手行为资料；修改现有文件时保留这些段落，新增对战演出暂不能套用旧单人对话逻辑。

现代文本调用的短名通常可复用，但槽参数、音乐参数和原始 opcode 需要按版本改。TH10 到 TH11 的编号变化尤其容易让 `ins_N` 变成另一条合法指令；采用默认 map 的名称可以避免一部分误移植。

## END

支持 TH10、11、12、128、13、14、15、16、17、18、19、20。TH10–19 的编码签名集合相同；这并不意味着所有游戏的等待、页数、音乐室和 staff 选择都逐作验证过。

TH16／18 的五行分页、ANM 加载、音乐、staff 流程见 [END](ending.md)。TH20 的主要写法变化：

- `textColor(color)` 改为 `textColor(color1, color2)`。
- `textAdd("|偏移,字距,注音")` 可以给下一行加注音，map 启用对应字符串兼容行为。
- `switchToStaff(file)` 开始使用文件名参数，TH18 按难度选 staff 的习惯不能用于任意文件跳转。

## STD

文件头和脚本指令是两个独立分界：TH06–09 用 `stage_name` 与四项 `bgm`，TH095 起用 `anm_path`。

| profile | 脚本写法 |
| --- | --- |
| TH06 | 位置关键帧 op0，雾 op1，朝向 op2；朝向计时 op3，雾计时 op4，暂停 op5；无跳转 |
| TH07 | 旧世代相机接口；新增跳转 op4、显式位置 op5、插值计时 op6、up/FOV 与中断 op31 |
| TH08、09 | 沿用 TH07 接口，新增摇动向量 op32／模式 op33、背景脚本 op34；仅这两个 profile 可用 strip quad |
| TH095 | 现代 opcode 编号；插值签名是 `Sbb--...`，即时间、两个字节模式和目标；sprite op14 两参，无中断标记 |
| TH10 | 插值改 `SU...`，即时间、一个模式与目标；sprite 仍两参，无中断标记 |
| TH103 | 在 TH10 基础上加入中断标记 op16 |
| TH11 | 加入 distortion op17 |
| TH12、125、128、13 | 加入 upTime op18 |
| TH14、143、15、16、165 | sprite op14 增加 layer；加入 ANM 全体中断 op19、绘制距离 op20 |
| TH17、18、185、19 | 加入 fovTime op21 |
| TH20 | 新增单整数 op22，行为尚未整理 |

TH07–09 的 `posTime(t,mode)` 与同帧 `pos(target)` 配对，TH10 起改成一次 `posTime(t,mode,target)`。旧插值枚举与现代枚举的数字不同，不能只照抄 mode 数字。TH095 两字节模式写法也不要用 TH10 的参数数目代替。

TH07–09 跳转底层用指令序号，现代 STD 用字节偏移。truth 的符号跳转会分别编码；日常修改不用手算偏移、指令长度或插入指令后的目标位置。
