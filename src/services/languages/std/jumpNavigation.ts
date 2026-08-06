/**
 * STD 跳转导航。
 *
 * thstd 的跳转目标是**裸字节偏移**，没有符号标签。要在文本里生成 `goto label`
 * 必须复刻 thstd 对每条指令的大小计算——任何一处对不上就是静默产出错误的
 * 跳转目标，而这类错误在游戏里才会暴露。
 *
 * 用户真正要的是"看得懂跳到哪"，这一点在编辑器层给就够了，而且**比真 label
 * 更好**：不改文件格式，`.dstd` 仍能经翻译回去喂给 thstd。
 */

export interface JumpLink {
  /** 1 起的源行号 */
  sourceLine: number
  /** 指令里写的偏移实参 */
  offset: number
  /** 跳转发生的时间点（thstd 的 jmp 同时带时间与偏移） */
  time: number
}

/**
 * 同时认两种写法：
 * - `jmp(time, offset)` —— 我们翻译后的形式，参数顺序随 ref/ECL 生态
 * - `ins_1(offset, time)` —— thstd 原始形式，两个参数顺序**相反**
 *
 * 这个顺序差异是 translator 里既有的特例（opcode 1 双向交换前两个实参），
 * 这里必须跟它保持一致，否则识别出的目标会张冠李戴。
 */
const JUMP_RE =
  /^\s*(?:(?:[+-]?\d+|@\w+)\s*:)?\s*(jmp|ins_1)\s*\(\s*(-?\d+)\s*,\s*(-?\d+)\s*\)/

/** 行首是注释就跳过——注释里的 jmp 不是跳转。 */
function isComment(line: string): boolean {
  const trimmed = line.trimStart()
  return trimmed.startsWith('//') || trimmed.startsWith('#')
}

export function findJumpTargets(text: string): JumpLink[] {
  const links: JumpLink[] = []

  text.split('\n').forEach((line, index) => {
    if (isComment(line)) return
    const match = JUMP_RE.exec(line)
    if (!match) return

    const [, mnemonic, first, second] = match
    // jmp(time, offset) vs ins_1(offset, time)
    const isTranslated = mnemonic === 'jmp'
    links.push({
      sourceLine: index + 1,
      time: Number(isTranslated ? first : second),
      offset: Number(isTranslated ? second : first)
    })
  })

  return links
}
