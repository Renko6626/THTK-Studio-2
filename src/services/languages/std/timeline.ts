/**
 * 时间线分析：把 `.dstd` / `.dmsg` 解析成"随时间发生了什么"。
 *
 * 两种格式都是**时间键控的指令流**——STD 是背景编排、MSG 是对话编排——所以同一套
 * 分析服务两者。
 *
 * 但**只有语义模型是共用的，文本形状不是**：thstd 输出 `720:` + `    jmp(...);`，
 * thmsg 输出 `@120` + `\ttextAdd(...)`。本文件早先写着「两边正则逐字相同」，那个
 * 前提是错的，直接导致对真实文件一行都匹配不上。逐方言的差异集中在
 * `TimelineLineSpec`，别再往回合并。
 *
 * ## 为什么不只是"列出时间标签"
 *
 * 时间点列表滚一遍文本也能看到。真正看不出来的是这四类（依据为 thtk 的
 * `formats_v2` 与我们 `.stdm` 里的说明，均一手）：
 *
 * | 类别 | 指令 | 时间线含义 |
 * | --- | --- | --- |
 * | 循环 | `1 jmp` | 「跳转…**并将当前时间设置为 time**」——是倒带，不只是控制流 |
 * | 外部入口 | `16 interruptLabel` | 「可以在脚本等待时**被 ECL 指令触发**」 |
 * | 持续区间 | `3/5/9/18/21 *Time`、`10/11 *Bezier` | 「在接下来的 **duration 帧内**平滑改变」——占区间不是点 |
 * | 无限等待 | `0 stop` | 「类似无限大的时间标签…随时可触发中断」——不是结束 |
 *
 * ## 插值是**非阻塞**的
 *
 * ExpHP 的 thpages 对 `stop` 的原文：
 *
 * > This is basically like putting an infinitely large time label in the script.
 * > **Time-interpolated values will update**, and interrupts can be triggered at any time
 *
 * 脚本都停了插值还在更新，说明 `*Time` / `*Bezier` 只是**启动**一个由帧时钟独立
 * 驱动的插值，不会卡住脚本。执行模型是：执行完某时间标签下的全部指令 → 等待
 * 时钟推进到下一个标签。
 *
 * 后果有两个，都必须在视图上体现，否则会误导：
 * 1. 一个 t=0、duration=60 的插值会**跨过** t=30 的时间标签继续进行；
 * 2. 多个插值可以**重叠**。
 *
 * 所以 span 要给出结束时间，并标出它是否越过了后续时间点。
 *
 * 光看一行 `ins_16(3)` 不可能知道那是个外部可触发的入口；看 `posTime(60, ...)`
 * 也不会意识到它一直持续到 60 帧后。这些才是时间线该给的东西。
 */

export type TimelineEventKind =
  | 'instant'
  /** 持续 duration 帧的插值 */
  | 'span'
  /** 跳转：会把当前时间设为 time */
  | 'loop'
  /** 外部（ECL）可触发的入口 */
  | 'interrupt'
  /** 无限等待，中断仍可触发 */
  | 'halt'

export interface TimelineEvent {
  /** 1 起的行号 */
  line: number
  /** 该指令所属的时间点 */
  time: number
  kind: TimelineEventKind
  name: string
  /** span：持续帧数。其余为 null */
  duration: number | null
  /** span：插值结束的时间点（time + duration）。其余为 null */
  endTime: number | null
  /**
   * span：该插值是否越过了后续的时间标签。
   *
   * 插值非阻塞，所以这完全正常——但它意味着"读到下一个时间点时这个动作还在进行中"，
   * 光看文本发现不了。
   */
  crossesNextLabel: boolean
  /** loop：跳转后设定的时间。其余为 null */
  jumpTime: number | null
  /** interrupt：中断编号。其余为 null */
  interruptId: number | null
}

export interface TimelineGroup {
  time: number
  /** 到下一个时间点的间隔；最后一组为 null */
  delta: number | null
  events: TimelineEvent[]
}

export interface TimelineAnalysis {
  groups: TimelineGroup[]
  /** 是否含 halt（无限等待） */
  hasHalt: boolean
}

/**
 * 各类特殊指令的 opcode / 名字，按版本分档。
 *
 * 与 `jumpNavigation.ts` 的 `jumpSpecFor` 同一模式：语义是声明式的，
 * 不在运行时猜。分档依据 truth 的 `core_mapfiles/std.rs`。
 */
interface StdTimelineSpec {
  /** 跳转指令名（我们的 translator 会把 ins_N 换成它）；该方言没有则为 null */
  jump: string | null
  /** 中断标签指令名；该方言没有则为 null */
  interrupt: string | null
  /** 无限等待指令名；该方言没有则为 null */
  halt: string | null
  /** 带 duration 的指令名 → duration 在第几个实参（0 起） */
  spans: Record<string, number>
  /**
   * 该方言的行形状。**两个工具的输出格式并不一样**，共用一条正则是行不通的：
   *
   * | | 时间标签 | 指令行 | 参数分隔 |
   * | --- | --- | --- | --- |
   * | thstd | `720:` 顶格独占一行 | `    jmp(...);` 4 空格缩进、**行尾分号** | `,` |
   * | thmsg | `@120` | `\ttextAdd(...)` TAB 缩进、无分号 | `;` |
   *
   * 依据分别是 `thstd.c` 与 `thmsg06.c` 的 dump 函数。此前这里只有一条按
   * `120:` + `name(args)` 写的正则，对真实 .dstd（分号）与 .dmsg（`@` 标签）
   * 都匹配不上，时间线是空的。
   */
  line: TimelineLineSpec
}

interface TimelineLineSpec {
  /**
   * 整行匹配，捕获组依次为：时间标签、指令名、实参串。
   *
   * 标签一律只捕获**数字部分**（`@120` 捕获 `120`，`+30:` 捕获 `+30`），
   * 于是下游只需一套推进逻辑，不必再按方言分支。
   */
  re: RegExp
  /** 实参分隔符 */
  argSep: string
}

/** thstd：`[720:] name(a, b);`，标签可独占一行，也允许手写时与指令同行。 */
const LINE_STD: TimelineLineSpec = {
  re: /^\s*(?:([+-]?\d+|@\w+)\s*:)?\s*(?:([A-Za-z_]\w*)\s*\((.*)\))?\s*;?\s*(?:\/\/.*)?$/,
  argSep: ','
}

/**
 * thmsg：`@120` 独占一行，指令行**必须有缩进**。
 *
 * 缩进这条不是洁癖：thmsg 会在文件开头输出一行 `header(0, 0)`，它顶格、也长得
 * 像调用。要求缩进正好把文件级的行（`header(...)`、`entry 0`）与指令区分开，
 * 而 thmsg 输出的每条指令都带 `\t`。
 */
const LINE_MSG: TimelineLineSpec = {
  re: /^(?:\s*@(\d+)\s*|[ \t]+([A-Za-z_]\w*)\s*\((.*)\)\s*(?:\/\/.*)?)$/,
  argSep: ';'
}

/**
 * th095 起（含 th14–th20）。
 *
 * `*Time` 与 `*Bezier` 的 duration 都是**第 0 个实参**——见 `.stdm` 的说明：
 * 「在接下来的 duration 帧内…」「在 duration 帧内，使用贝塞尔曲线…」。
 */
const SPEC_095_PLUS: StdTimelineSpec = {
  jump: 'jmp',
  interrupt: 'interruptLabel',
  halt: 'stop',
  spans: {
    posTime: 0,
    facingTime: 0,
    fogTime: 0,
    upTime: 0,
    fovTime: 0,
    posBezier: 0,
    facingBezier: 0
  },
  line: LINE_STD
}

/** MSG 没有跳转与中断，只有时间点。 */
const SPEC_MSG: StdTimelineSpec = {
  jump: null,
  interrupt: null,
  halt: null,
  spans: {},
  line: LINE_MSG
}

export type TimelineDialect = 'std' | 'msg'

export function specFor(dialect: TimelineDialect): StdTimelineSpec {
  return dialect === 'std' ? SPEC_095_PLUS : SPEC_MSG
}

/**
 * 该方言里**带时间线语义**的指令名。语法高亮拿它把这些指令标成 builtin，
 * 与时间线面板共用同一份来源——不再各存一份名字列表。
 */
export function timelineKeywords(dialect: TimelineDialect): string[] {
  const spec = specFor(dialect)
  return [spec.jump, spec.interrupt, spec.halt, ...Object.keys(spec.spans)].filter(
    (name): name is string => Boolean(name)
  )
}

function parseArgs(args: string, sep: string): number[] {
  const trimmed = args.trim()
  if (!trimmed) return []
  return trimmed.split(sep).map((a) => Number(a.trim()))
}

/**
 * 解析时间线。
 *
 * 行的形状由方言决定（见 `TimelineLineSpec`）：thstd 是 `720:` + `名字(...);`，
 * thmsg 是 `@120` + `\t名字(...)`。
 *
 * 时间标签支持绝对（`60:`、`@120`）与相对（`+30:`）两种写法——两个工具的 dump
 * 都用绝对，但用户手写时可能用相对，两种都认。`@label:` 这种符号标签不参与时间
 * 推进（与 MSG 的 `@120` 不冲突：后者被捕获为裸数字 `120`）。
 */
export function analyzeTimeline(text: string, dialect: TimelineDialect): TimelineAnalysis {
  const spec = specFor(dialect)
  const groups: TimelineGroup[] = []
  let currentTime = 0
  let currentGroup: TimelineGroup | null = null
  let hasHalt = false

  const ensureGroup = (time: number): TimelineGroup => {
    if (currentGroup && currentGroup.time === time) return currentGroup
    const group: TimelineGroup = { time, delta: null, events: [] }
    groups.push(group)
    currentGroup = group
    return group
  }

  text.split('\n').forEach((raw, index) => {
    const match = spec.line.re.exec(raw)
    if (!match) return
    const [, label, name, args] = match

    if (label !== undefined) {
      if (label.startsWith('+')) {
        currentTime += Number(label.slice(1))
      } else if (!label.startsWith('@')) {
        currentTime = Number(label)
      }
    }
    if (name === undefined) return

    const values = parseArgs(args ?? '', spec.line.argSep)
    const group = ensureGroup(currentTime)

    let kind: TimelineEventKind = 'instant'
    let duration: number | null = null
    let jumpTime: number | null = null
    let interruptId: number | null = null

    if (name === spec.jump && values.length >= 2) {
      kind = 'loop'
      // 我们的 translator 已把 STD 的 jmp 归一成 (time, offset)
      jumpTime = values[0]
    } else if (name === spec.interrupt && values.length >= 1) {
      kind = 'interrupt'
      interruptId = values[0]
    } else if (name === spec.halt) {
      kind = 'halt'
      hasHalt = true
    } else if (name in spec.spans) {
      const argIndex = spec.spans[name]
      const value = values[argIndex]
      if (Number.isFinite(value)) {
        kind = 'span'
        duration = value
      }
    }

    group.events.push({
      line: index + 1,
      time: currentTime,
      kind,
      name,
      duration,
      endTime: duration === null ? null : currentTime + duration,
      crossesNextLabel: false, // 下面统一回填：此时还不知道后续有哪些时间点
      jumpTime,
      interruptId
    })
  })

  for (let i = 0; i < groups.length - 1; i += 1) {
    groups[i].delta = groups[i + 1].time - groups[i].time
  }

  // 插值非阻塞，所以它可能越过后续时间标签。这一步要等全部时间点都收集完
  // 才能判断，故放在分组之后统一回填。
  for (let i = 0; i < groups.length; i += 1) {
    const nextTime = groups[i + 1]?.time
    if (nextTime === undefined) continue
    for (const event of groups[i].events) {
      if (event.endTime !== null && event.endTime > nextTime) {
        event.crossesNextLabel = true
      }
    }
  }

  return { groups, hasHalt }
}

/** 根据文件路径判断该用哪套方言；不是时间线格式则返回 null。 */
export function dialectForPath(path: string | null | undefined): TimelineDialect | null {
  const lower = String(path || '').toLowerCase()
  if (lower.endsWith('.dstd')) return 'std'
  if (lower.endsWith('.dmsg')) return 'msg'
  return null
}
