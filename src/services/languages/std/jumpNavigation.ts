/**
 * STD 跳转导航。
 *
 * 只做**只读导航**：算出跳转指向哪一行，不把 label 写回文件——真 label 要求
 * 往 `.dstd` 里写入 thstd 不认识的语法，那文件就喂不回去了。
 *
 * ## 语义全部来自 ExpHP/truth，是**声明式**的，没有运行时推断
 *
 * truth 的 `src/core_mapfiles/std.rs` 用签名里的 `o` 标记"这个参数是跳转目标"，
 * 并用 `IntrinsicInstrKind::Jmp` 标记跳转内建；`src/formats/std.rs` 的
 * `LanguageHooks` 再声明该值怎么换算成字节偏移。两处合起来就是下表：
 *
 * | 版本 | hooks | 跳转 opcode | 签名 | 参数含义 |
 * | --- | --- | --- | --- | --- |
 * | th06 | StdHooks06 | 无跳转 | — | — |
 * | th07–th09 | StdHooks06 | **4** | `ot_` | **指令序号**，`decode_label = bits * 20` |
 * | th095–th20 | StdHooks10 | **1** | `ot` | **字节偏移**，未 override `decode_label` → 脚本段起点起算 |
 *
 * `StdHooks06::decode_label` 之所以乘 20，是因为 v0 的指令**定长 20 字节**
 * （8 字节头 + 3 个 4 字节参数，thstd 按 `sizeof(uint32_t) * 3` 读）。
 *
 * ## 指令字节大小
 *
 * `instr_header_size() = 8`（truth）与 `std_instr_t = {u32 time; u16 type; u16 size}`
 * （thtk）一致。v1/v2 变长：编译时 `instr->size = sizeof(*instr)` 起步、每个参数
 * `+= sizeof(int32_t)`，而 `formats_v1/v2` 只用 `S`/`f`/`C` 三种格式字符，都是 4 字节。
 * 于是 **8 + 4 × 参数个数**；参数个数直接数 dump 里括号内的逗号，不需要 mapfile。
 */

/** 某一档 STD 的跳转规格。null 表示该版本没有跳转指令。 */
export interface StdJumpSpec {
  opcode: number
  /** 助记符（我们的翻译器会把 `ins_N` 换成它） */
  mnemonic: string
  /** 参数值 → 字节偏移的倍率：v0 存指令序号（×20），v1/v2 直接是字节偏移（×1） */
  offsetScale: number
  /** 指令定长时的字节数；null 表示变长（8 + 4×参数个数） */
  fixedInstrBytes: number | null
}

const HOOKS_06: StdJumpSpec = {
  opcode: 4,
  mnemonic: 'jmp',
  offsetScale: 20,
  fixedInstrBytes: 20
}

const HOOKS_10: StdJumpSpec = {
  opcode: 1,
  mnemonic: 'jmp',
  offsetScale: 1,
  fixedInstrBytes: null
}

/**
 * 按游戏版本取跳转规格。版本号是 thtk 的数字形式（`17` 而非 `th17`）。
 *
 * 分档与 truth 的 `core_signatures()` 一致：th06 单列（无跳转）、th07–th09 一档、
 * th095 起一档。th19/th20 truth 尚未支持，但它们同属 `StdHooks10` 那一档——
 * thtk 的 `formats_v2` 覆盖 th14–th20 且无 per-version 分支，可以放心归入。
 */
export function jumpSpecFor(version: number): StdJumpSpec | null {
  if (version === 6) return null
  if (version === 7 || version === 8 || version === 9) return HOOKS_06
  return HOOKS_10
}

export interface StdInstruction {
  /** 1 起的行号 */
  line: number
  /** 相对脚本段起点的字节偏移 */
  offset: number
  opcode: number
  argCount: number
}

export interface JumpLink {
  /** 1 起的源行号 */
  sourceLine: number
  /** 指令里写的偏移实参（未换算） */
  rawOffset: number
  /** 换算后的字节偏移 */
  byteOffset: number
  time: number
  /** 目标行号；实参没落在指令边界上时为 null */
  targetLine: number | null
}

export interface StdAnalysis {
  instructions: StdInstruction[]
  jumps: JumpLink[]
}

const INSTR_HEADER_BYTES = 8
const ARG_BYTES = 4

/** `[时间标签:] name(args)` 或 `[时间标签:] ins_N(args)`，可带行尾注释 */
const INSTR_RE =
  /^\s*(?:(?:[+-]?\d+|@\w+)\s*:)?\s*(?:ins_(\d+)|([A-Za-z_]\w*))\s*\((.*)\)\s*(?:\/\/.*)?$/

function isSkippable(line: string): boolean {
  const trimmed = line.trimStart()
  return (
    trimmed === '' ||
    trimmed.startsWith('//') ||
    trimmed.startsWith('#') ||
    /^[A-Z_]+:\s*$/.test(trimmed) ||
    /^(?:[+-]?\d+|@\w+)\s*:\s*$/.test(trimmed)
  )
}

function countArgs(args: string): number {
  const trimmed = args.trim()
  return trimmed ? trimmed.split(',').length : 0
}

/**
 * 扫描可读 `.dstd`，得到每条指令的字节偏移与全部跳转的目标行。
 *
 * @param version thtk 的数字版本号（`17`）。决定跳转 opcode 与偏移换算方式。
 */
export function analyzeStd(text: string, version: number): StdAnalysis {
  const spec = jumpSpecFor(version)
  const instructions: StdInstruction[] = []
  const pending: Omit<JumpLink, 'targetLine'>[] = []
  let offset = 0

  text.split('\n').forEach((line, index) => {
    if (isSkippable(line)) return
    const match = INSTR_RE.exec(line)
    if (!match) return

    const [, insN, name, args] = match
    const argCount = countArgs(args)
    const opcode = insN !== undefined ? Number(insN) : -1
    const isJump =
      spec !== null &&
      (name === spec.mnemonic || (insN !== undefined && Number(insN) === spec.opcode))

    instructions.push({
      line: index + 1,
      offset,
      opcode: isJump && opcode < 0 ? spec!.opcode : opcode,
      argCount
    })

    if (isJump && argCount >= 2) {
      const parts = args.split(',').map((a) => Number(a.trim()))
      // 原始 `ins_N(offset, time)` 与翻译后的 `jmp(time, offset)` 顺序相反。
      // truth 的签名是 `ot`（offset 在前），与 thstd 的原始顺序一致；
      // 我们的 translator 对该 opcode 做了双向交换，故两种都要认。
      const translated = name === spec!.mnemonic
      const rawOffset = translated ? parts[1] : parts[0]
      const time = translated ? parts[0] : parts[1]
      pending.push({
        sourceLine: index + 1,
        rawOffset,
        byteOffset: rawOffset * spec!.offsetScale,
        time
      })
    }

    offset += spec?.fixedInstrBytes ?? INSTR_HEADER_BYTES + ARG_BYTES * argCount
  })

  const lineByOffset = new Map(instructions.map((i) => [i.offset, i.line]))
  const jumps: JumpLink[] = pending.map((jump) => ({
    ...jump,
    // 没落在指令边界上就不给目标——宁可不显示，也不要指向错的行
    targetLine: lineByOffset.get(jump.byteOffset) ?? null
  }))

  return { instructions, jumps }
}

/** 只要跳转，供不关心指令列表的调用方使用。 */
export function findJumpTargets(text: string, version: number): JumpLink[] {
  return analyzeStd(text, version).jumps
}
