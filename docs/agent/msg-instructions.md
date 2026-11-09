# MSG 指令参考

人工维护的关卡对话指令说明。文件结构和例子见 [MSG 编程介绍](msg.md)。支持 TH06、TH07、TH08、TH09、TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20；mission.msg 不在范围内。小数作品采用工具链编号，如 TH12.8 为 TH128。

先确定当前任务的目标版本，再选该版本明确列出的行。每行的版本范围独立，不继承其他行或其他作品的行为；未列出的版本不适用。只确认签名、尚未整理行为的条目会直接说明。项目默认版本仅供参考。

int、float、string 是源码类型，参数按调用顺序传入；槽、颜色、布尔值和资源编号也用整数。调用名采用默认 map 的名称，ins_N 为十进制原始编号。项目可自定义 map，遇到名称或签名不符时用现有工具核对；签名一致也不证明游戏行为一致。

## TH10 及之后：逐条适用版本

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 0 | `end()` | 结束并释放当前对话 VM；关卡结算使用另一条指令 | 无 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15 | 1 | `playerShow()` | 显示玩家侧立绘 | 无有效参数，填充由编译器处理 |
| TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 1 | `playerShow(slot)` | 显示玩家侧立绘 | int slot：玩家侧立绘槽，资源来自自机 ANM 配置 |
| TH10、TH11、TH12 | 2 | `bossShow()` | 显示敌方立绘 | 无有效参数，填充由编译器处理 |
| TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 2 | `bossShow(slot)` | 显示敌方立绘 | int slot：敌方立绘槽，资源来自关卡 ANM 配置 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH165、TH17、TH185、TH19、TH20 | 3 | `textboxShow()` | 文本框显示操作 | 无；现代气泡的逐作初始化细节未完整整理 |
| TH16、TH18 | 3 | `textboxShow()` | 旧式文本框操作，不负责现代气泡初始化 | 无 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17 | 4 | `playerHide()` | 隐藏玩家侧立绘 | 无 |
| TH18 | 4 | `playerHide(slot)` | 隐藏玩家侧指定槽的立绘 | int slot：本侧槽号；map 的 PortraitSide 注解不表示这里是左右侧 |
| TH185、TH19、TH20 | 4 | `playerHide(value)` | 隐藏玩家侧立绘 | int value：map 标注 PortraitSide；槽选择细节未逐作整理 |
| TH10、TH11、TH12、TH128、TH13 | 5 | `bossHide()` | 隐藏敌方立绘 | 无 |
| TH14、TH143、TH15、TH16、TH165、TH17 | 5 | `bossHide(slot)` | 隐藏敌方指定立绘 | int slot：敌方立绘选择参数 |
| TH18 | 5 | `bossHide(slot)` | 隐藏敌方指定槽的立绘 | int slot：本侧槽号；map 的 PortraitSide 注解不表示这里是左右侧 |
| TH185、TH19、TH20 | 5 | `bossHide(value)` | 隐藏敌方立绘 | int value：map 标注 PortraitSide；槽选择细节未逐作整理 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 6 | `textboxHide()` | 隐藏文字与气泡／文本框 | 无 |
| TH10、TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17 | 7 | `speakerPlayer()` | 指定玩家侧为说话者 | 无 |
| TH18 | 7 | `speakerPlayer(slot)` | 指定玩家侧某槽为说话者 | int slot：本侧槽号；map 的 PortraitSide 注解不表示左右侧 |
| TH185、TH19、TH20 | 7 | `speakerPlayer(value)` | 指定玩家侧为说话者 | int value：map 标注 PortraitSide；具体槽选择未逐作整理 |
| TH10、TH11、TH12、TH128、TH13 | 8 | `speakerBoss()` | 指定敌方为说话者 | 无 |
| TH14、TH143、TH15、TH16、TH165、TH17 | 8 | `speakerBoss(slot)` | 指定敌方立绘为说话者 | int slot：敌方立绘选择参数 |
| TH18 | 8 | `speakerBoss(slot)` | 指定敌方某槽为说话者 | int slot：本侧槽号；map 的 PortraitSide 注解不表示左右侧 |
| TH185、TH19、TH20 | 8 | `speakerBoss(value)` | 指定敌方为说话者 | int value：map 标注 PortraitSide；具体槽选择未逐作整理 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 9 | `speakerNone()` | 旁白，不突出任何说话者 | 无 |
| TH10 | 9 | `skippable(enabled)` | 设置长按快进 | int enabled：0 禁用、1 启用，可写 false／true |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 10 | `skippable(enabled)` | 设置长按快进 | int enabled：0 禁用、1 启用，可写 false／true |
| TH10 | 10 | `textPause(maxFrames)` | 等待输入或超时 | int maxFrames：最长等待帧数 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH165、TH17、TH185、TH19、TH20 | 11 | `textPause(maxFrames)` | 等待输入或超时 | int maxFrames：最长等待帧数；分页与脚本时钟细节未逐作整理 |
| TH16、TH18 | 11 | `textPause(maxFrames)` | 等待输入或超时；阅读时冻结脚本时钟，放行后下一条文本从新页开始 | int maxFrames：最长等待帧数，不必再加到后续时间标签 |
| TH10 | 11 | `eclResume()` | 放行等待对话的 ECL | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH165、TH17、TH185、TH19、TH20 | 12 | `eclResume()` | 放行等待对话的 ECL | 无；具体信号时序未逐作整理 |
| TH16、TH18 | 12 | `eclResume()` | 向等待对话的 ECL 发出短脉冲，不能当成永久开关 | 无 |
| TH10 | 12 | `playerFace(face)` | 切换玩家侧表情 | int face：表情号 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17 | 13 | `playerFace(face)` | 切换玩家侧表情 | int face：表情号 |
| TH18 | 13 | `playerFace(face, slot)` | 切换玩家侧指定槽的表情 | int face：表情号；int slot：本侧槽号；通过 ANM 中断 17 + face 切换 |
| TH185、TH19、TH20 | 13 | `playerFace(face, slot)` | 切换玩家侧表情 | int face、slot：map 依次标注 ANM 脚本和立绘槽；具体 ANM 协议未逐作整理 |
| TH10 | 13 | `bossFace(face)` | 切换敌方表情 | int face：表情号 |
| TH11、TH12、TH128、TH13 | 14 | `bossFace(face)` | 切换敌方表情 | int face：表情号 |
| TH14、TH143、TH15、TH16、TH165、TH17 | 14 | `bossFace(face, slot)` | 切换敌方指定立绘的表情 | int face：表情号；int slot：敌方立绘选择参数 |
| TH18 | 14 | `bossFace(face, slot)` | 切换敌方指定槽的表情 | int face：表情号；int slot：本侧槽号；通过 ANM 中断 17 + face 切换 |
| TH185、TH19、TH20 | 14 | `bossFace(face, slot)` | 切换敌方表情 | int face、slot：map 依次标注 ANM 脚本和立绘槽；具体 ANM 协议未逐作整理 |
| TH10 | 14 | `textLine1(text)` | 直接设置第一行文本 | string text：文字 |
| TH10 | 15 | `textLine2(text)` | 直接设置第二行文本 | string text：文字 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 15 | `textLine1(text)` | 直接设置第一行文本 | string text：文字；日常分页对话使用 textAdd |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 16 | `textLine2(text)` | 直接设置第二行文本 | string text：文字 |
| TH10 | 16 | `textAdd(text)` | 写下一行对话 | string text：文字；不使用后续版本的注音兼容写法 |
| TH11 | 17 | `textAdd(text)` | 写下一行对话 | string text：文字；尚无后续注音字符串兼容 |
| TH12、TH128、TH13、TH14、TH143、TH15、TH165、TH17、TH185、TH19、TH20 | 17 | `textAdd(text)` | 写下一行对话；可在正文前设置注音 | string text：文字或注音前缀；逐作分页行数未完整整理 |
| TH16、TH18 | 17 | `textAdd(text)` | 写下一行对话，每页两行；第三条先清掉上一页；支持注音 | string text：文字或注音前缀 |
| TH10 | 17 | `textClear()` | 清除已有文本 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 18 | `textClear()` | 清除已有文本 | 无 |
| TH10 | 18 | `musicBoss()` | 播放 boss 曲并显示曲名 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH20 | 19 | `musicBoss()` | 播放 boss 曲并显示曲名 | 无 |
| TH185、TH19 | 19 | `musicBoss(index)` | 选择 boss 曲并显示曲名 | int index：曲目编号 |
| TH10 | 19 | `intro()` | 显示 boss 名字与称号 | 无 |
| TH11、TH12、TH128、TH13 | 20 | `intro()` | 显示 boss 名字与称号 | 无 |
| TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH20 | 20 | `intro(who)` | 显示指定 boss 的名字与称号 | int who：目标选择参数，沿用目标文件的编号 |
| TH19 | 20 | `intro()` | 名牌相关操作，逐作效果未整理 | 无有效参数，map 中该字段是填充，不能套用带参调用 |
| TH10 | 20 | `stageEnd()` | 进入关卡结算／下一关流程，需与 ECL 配合 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 21 | `stageEnd()` | 进入关卡结算／下一关流程，需与 ECL 配合 | 无 |
| TH10 | 21 | `musicEnd()` | 按该作默认时长淡出音乐 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH185、TH19、TH20 | 22 | `musicEnd()` | 按该作默认时长淡出音乐 | 无；具体时长未逐作整理 |
| TH18 | 22 | `musicEnd()` | 淡出音乐：通常 8 秒，六面 2 秒 | 无 |
| TH10 | 22 | `playerShake()` | 玩家侧立绘抖动 | 无 |
| TH10 | 23 | `bossShake()` | 敌方立绘抖动 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH185、TH19、TH20 | 23 | `playerShake()` | 玩家侧立绘抖动 | 无；具体作用槽未逐作整理 |
| TH18 | 23 | `playerShake()` | 使玩家侧 0、1 号立绘抖动 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH185、TH19、TH20 | 24 | `bossShake()` | 敌方立绘抖动 | 无；具体作用槽未逐作整理 |
| TH18 | 24 | `bossShake()` | 使敌方 0、1 号立绘抖动 | 无 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 25 | `textOffsetY(dy)` | 调整文本纵向偏移 | int dy：偏移量，沿用原文件坐标尺度 |
| TH11、TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 26 | `ins_26()` | 文本样式／标志相关，完整行为未整理 | 无 |
| TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH185、TH19、TH20 | 27 | `musicFade(seconds)` | 音乐淡出相关；各作读取参数的行为未全部核对 | float seconds：当前 map 接受的时长参数，单位秒 |
| TH18 | 27 | `musicFade(seconds)` | 按指定秒数淡出音乐 | float seconds：淡出秒数 |
| TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH185、TH19、TH20 | 28 | `bubblePos(x, y)` | 设置气泡位置 | float x、y：位置；引擎坐标缩放未逐作整理 |
| TH18 | 28 | `bubblePos(x, y)` | 设置气泡位置，引擎按两倍坐标使用 | float x、y：位置，沿用原文件的坐标尺度 |
| TH128、TH13、TH14、TH143、TH15、TH165、TH17、TH18、TH185、TH19、TH20 | 29 | `bubbleType(type)` | 选择气泡 ANM 样式 | int type：样式号，实际外观取决于目标气泡 ANM，不沿用其他作品的外观编号 |
| TH16 | 29 | `bubbleType(type)` | 选择气泡 ANM 样式 | int type：0 圆角矩形、1 尖刺、2 思考云、3 说话云、4–7 附身红框；对应本作气泡 ANM |
| TH128 | 30 | `routeSelect()` | 第一关路线选择 | 无 |
| TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20 | 30 | `routeSelect()` | 同名签名存在，实际用途未逐作整理 | 无 |
| TH13、TH14、TH15、TH16、TH17 | 31 | `showEnemy1(value)` | 旧资料描述为显示敌方 1 号立绘；逐作细节未完整确认 | int value：当前 map 要求传入，参数用途未完整确认 |
| TH18、TH185 | 31 | `showEnemy1()` | 旧资料描述为显示敌方 1 号立绘；逐作细节未完整确认 | 无 |
| TH143、TH165 | 31 | `ins_31(value)` | 行为未整理 | int value：用途未整理 |
| TH14、TH15、TH16、TH17、TH185 | 32 | `modern32(value)` | 气泡朝向相关，逐作行为未完整整理 | int value：用途与取值未逐作确认 |
| TH18 | 32 | `modern32(side)` | 只改气泡朝向，不改变立绘明暗 | int side：0 左、1 右、2 无人 |
| TH143、TH165 | 32 | `ins_32(value)` | 行为未整理 | int value：用途未整理 |
| TH19、TH20 | 32 | `__unknown_A(value)` | 行为未整理 | int value：不能套用 TH18 的方向含义 |
| TH16、TH18 | 33 | `portraitDarken(side, slot)` | 单独变暗立绘 | int side：0 左、1 右；int slot：立绘槽 |
| TH17、TH185、TH19、TH20 | 33 | `portraitDarken(a, b)` | 单独变暗立绘，参数的逐作含义未完整整理 | int a、b：两个整数，不能仅凭同名签名套用 TH18 的侧别／槽号解释 |
| TH143、TH165 | 33 | `tutorial(value)` | 教程相关，具体行为未整理 | int value：用途未完整整理 |
| TH16、TH18 | 34 | `portraitHighlight(side, slot)` | 单独高亮立绘 | int side：0 左、1 右；int slot：立绘槽 |
| TH17、TH185、TH19、TH20 | 34 | `portraitHighlight(a, b)` | 单独高亮立绘，参数的逐作含义未完整整理 | int a、b：两个整数，不能仅凭同名签名套用 TH18 的侧别／槽号解释 |
| TH16、TH17、TH18 | 35 | `lightsOut()` | 用黑幕遮住关卡背景和敌机 | 无 |
| TH185、TH19、TH20 | 35 | `lightsOut()` | 同名黑幕相关指令，具体遮挡对象未逐作核对 | 无 |
| TH18 | 36 | `store()` | 标记对话后进入能力卡商店 | 无 |
| TH185、TH19、TH20 | 36 | `store()` | 同名签名存在，实际用途未逐作整理；不能套用 TH18 商店行为 | 无 |

## TH06–TH09：逐条适用版本

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH06、TH07、TH08、TH09 | 0 | `end()` | 结束对话 | 无 |
| TH06、TH07、TH08 | 1 | `show(who, script)` | 创建立绘 | int who：槽号；int script：ANM 脚本号 |
| TH09 | 1 | `show(who)` | 创建立绘，脚本选择由游戏处理 | int who：槽号；没有第二个有效参数 |
| TH06、TH07、TH08、TH09 | 2 | `face(who, sprite)` | 切换立绘表情 | int who：槽号；int sprite：ANM sprite 号 |
| TH06、TH07 | 3 | `textSet(color, line, text)` | 设置指定文本行 | int color：0 浅蓝、1 浅红；int line：0 上行、1 下行；string text：文字 |
| TH08、TH09 | 3 | `textSet(color, line, text)` | 设置指定文本行 | int color：本作文本颜色枚举；int line：行号；string text：文字 |
| TH06、TH07、TH08、TH09 | 4 | `textPause(maxFrames)` | 等待输入或超时 | int maxFrames：最长等待帧数 |
| TH06、TH07 | 5 | `anmInterrupt(who, interrupt)` | 向立绘发送 ANM 中断 | int who：目标槽；int interrupt：常用 1 登场、3 高亮、4 变暗、5 退场 |
| TH08、TH09 | 5 | `anmInterrupt(who, interrupt)` | 向立绘发送 ANM 中断 | int who：目标槽；int interrupt：中断号，具体协议按本作 ANM |
| TH06、TH07、TH08、TH09 | 6 | `eclResume()` | 放行 ECL | 无 |
| TH06、TH07、TH08、TH09 | 7 | `music(index)` | 按编号操作音乐 | int index：音乐编号，对应曲目依目标游戏 |
| TH06、TH07、TH08 | 8 | `intro(color, line, text)` | 写 boss 名字与称号 | int color：文本颜色；int line：行号；string text：文字 |
| TH09 | 8 | `intro()` | boss 名牌相关，无文本参数 | 无 |
| TH06、TH07、TH08 | 9 | `stageResults()` | 关卡结果相关 | 无有效参数，填充由编译器处理 |
| TH09 | 9 | `stageResults(value)` | 关卡结果相关 | int value：用途未完整整理 |
| TH06、TH07、TH08、TH09 | 10 | `stop()` | 停止后续脚本执行，恢复条件未逐作整理；不同于释放 VM | 无 |
| TH06、TH07、TH08、TH09 | 11 | `stageEnd()` | 进入关卡结束流程 | 无 |
| TH06、TH07、TH08、TH09 | 12 | `musicEnd()` | 音乐结束／淡出 | 无；时长依目标游戏 |
| TH06、TH07、TH08、TH09 | 13 | `skippable(enabled)` | 设置快进标志 | int enabled：0 禁用、1 启用 |
| TH07、TH08、TH09 | 14 | `screenFade()` | 画面渐变相关，方向和细节未逐作整理 | 无 |
| TH08 | 15 | `focusSetEx(who, s0, s1, s2, s3)` | 同时换多个立绘表情并指定说话者 | int who：说话者槽；int s0、s1、s2、s3：四个 sprite 号 |
| TH09 | 15 | `focusSetEx(who, s0, s1)` | 同时换立绘表情并指定说话者 | int who：说话者，-1 为无人说话；int s0、s1：两个 sprite 号 |
| TH08、TH09 | 16 | `textAdd(text)` | 写下一行文本 | string text：文字 |
| TH08、TH09 | 17 | `focusSet(who, sprite)` | 切换说话者和表情 | int who：槽号；int sprite：表情 sprite 号 |
| TH08、TH09 | 18 | `msg_flag_textbox_visible(enabled)` | 文本框可见标志，组合效果未完整整理 | int enabled：布尔标志 |
| TH08 | 19 | `optionA(text)` | 设置第一条路线候选文字 | string text：候选文字，保留原关卡的分支配合 |
| TH08 | 20 | `optionB(text)` | 设置第二条路线候选文字 | string text：候选文字 |
| TH08 | 21 | `optionPause(maxFrames)` | 等待路线选择 | int maxFrames：自动选择前最长等待帧数 |
| TH08 | 22 | `option22()` | 路线选择相关，行为未整理 | 无 |
| TH09 | 23 | `__unknown_player_A(side)` | 行为未整理 | int side：map 标注左右侧，运行含义未整理 |
| TH09 | 24 | `__unknown_timer_set_1_A()` | 行为未整理 | 无 |
| TH09 | 25 | `__unknown_gui_anm_A()` | 行为未整理 | 无 |
| TH09 | 26 | `text_speaker_color(color)` | 说话者颜色相关，具体效果未整理 | int color：本作文本颜色枚举 |
| TH09 | 28 | `std_interrupt(id)` | 背景中断相关，调用时序按原文件 | int id：中断编号 |

TH06／TH07 的槽为 LeftPortrait=0、RightPortrait=1；TH08 为 LeftPortraitA=0、LeftPortraitB=1、RightPortraitA=2、RightPortraitB=3；TH09 为 Portrait0–Portrait3。TH06／TH07 明文、TH08 固定 XOR、TH09 加速 XOR 均由编译器处理。

## TH185／TH19 的额外指令

双下划线名称是现有默认 map 的名称；行为未整理的条目不能仅凭名称推断用途。TH19 的相对侧也不能套成单人对话的固定左右侧。

| 适用版本 | 编号 | 调用 | 作用 | 参数类型和含义 |
| --- | --- | --- | --- | --- |
| TH185、TH19 | 37 | `__initialize_infobox()` | 说明界面初始化相关，细节未整理 | 无 |
| TH185、TH19 | 38 | `__focus_infobox()` | 说明界面焦点相关，细节未整理 | 无 |
| TH185、TH19 | 39 | `__delete_infobox()` | 说明界面移除相关，细节未整理 | 无 |
| TH185、TH19 | 40 | `__unknown_text_A(text)` | 文本指令，显示位置和分页行为未整理 | string text：文字 |
| TH185、TH19 | 41 | `__unknown_text_B(text)` | 文本指令，显示位置和分页行为未整理 | string text：文字 |
| TH19 | 42 | `__focus_current_side(value)` | 相对侧说话者相关，完整行为未整理 | int value：map 标注 PortraitSide，具体含义未整理 |
| TH19 | 43 | `__focus_opposite_side(value)` | 相对侧说话者相关，完整行为未整理 | int value：map 标注 PortraitSide，具体含义未整理 |
| TH19 | 44 | `__unknown_position_A(x, y)` | 坐标相关，具体对象与尺度未整理 | float x、y：两个坐标类数值 |
| TH19 | 45 | `__unknown_position_B(x, y)` | 坐标相关，具体对象与尺度未整理 | float x、y：两个坐标类数值 |
| TH19 | 46 | `__portrait_current_side(face, slot)` | 相对侧表情相关，完整行为未整理 | int face、slot：map 依次标注 ANM 脚本、立绘槽 |
| TH19 | 47 | `__portrait_opposite_side(face, slot)` | 相对侧表情相关，完整行为未整理 | int face、slot：map 依次标注 ANM 脚本、立绘槽 |
| TH19 | 48 | `__initialize_current_side(slot)` | 相对侧立绘初始化相关，完整行为未整理 | int slot：map 标注立绘槽 |
| TH19 | 49 | `__initialize_opposite_side(slot)` | 相对侧立绘初始化相关，完整行为未整理 | int slot：map 标注立绘槽 |
| TH19 | 50 | `__delete_current_side(value)` | 相对侧立绘删除相关，完整行为未整理 | int value：map 标注 PortraitSide，具体含义未整理 |
| TH19 | 51 | `__delete_opposite_side(value)` | 相对侧立绘删除相关，完整行为未整理 | int value：map 标注 PortraitSide，具体含义未整理 |
| TH19 | 52 | `__gui_unknown_A()` | 行为未整理 | 无 |
| TH19 | 53 | `__player_unknown_A()` | 行为未整理 | 无 |
| TH19 | 54 | `__player_unknown_B()` | 行为未整理 | 无 |
| TH19 | 55 | `__spawn_msg_boss_enemy()` | 行为未整理 | 无 |
| TH19 | 56 | `__gui_unknown_B()` | 行为未整理 | 无 |

## 已限定版本的配合说明

- TH16／TH18：立绘 ANM 中断为 17 + 表情号切表情，1 隐藏、2 高亮、3 变暗、7 抖动。MSG 不能任意指定立绘 ANM 路径。
- TH16／TH18：ECL 的 dialogRead（518）启动对话后自身继续执行；dialogWait（519）等待 MSG 放行信号或对话结束。boss 中途入场时，在正确时间发 eclResume，并把 ECL 移动接在相应等待后。
- TH12、TH128、TH13、TH14、TH143、TH15、TH16、TH165、TH17、TH18、TH185、TH19、TH20：注音可写成 `textAdd("|0,11,あす")`，依次为横向偏移、字距、注音；修饰下一行且不占正文行。加密、填充与字符串尾部兼容由编译器处理。
- TH19：修改已有文件时保留 meta.header 和入口 flags。这些文件结构不能推广到其他版本。
