/**
 * ECL 词表（eclmap）相关类型。
 *
 * 这里有**三个**形状相近但不相同的类型，不要合并：
 * 1. `EclMapSemanticData` —— Rust `get_ecl_map_semantics` 原样返回的
 * 2. `LoadedEclSemanticData` —— 加载器在 1 之上补了解析路径与错误
 * 3. `NormalizedEclSemanticData` —— 归一化后存进作用域表的，丢掉了 globals、
 *    并把指令名并进了 builtins
 *
 * 对应 `src-tauri/src/modules/ecl/map_parser.rs` 与
 * `src/services/languages/ecl/dynamic-vocabulary.js`。
 */

/**
 * 对应 map_parser.rs 的 `EclMapInstructionParameter`。
 *
 * ⚠️ Rust 侧字段是 `type_name`，但带 `#[serde(rename = "type")]`——
 * 序列化出来叫 `type` 而不是 `typeName`。
 */
export interface EclInstructionParameter {
  name: string
  type: string
}

/** 对应 map_parser.rs 的 `EclMapInstructionSpec`（`rename_all = "camelCase"`） */
export interface EclInstructionSpec {
  opcode: number
  name: string
  section: string | null
  signature: string | null
  params: EclInstructionParameter[]
}

/** 对应 map_parser.rs 的 `EclMapGlobalVar`（`rename_all = "camelCase"`） */
export interface EclGlobalVar {
  id: number
  name: string
  /** "int" | "float" | "unknown"，Rust 侧是裸 String */
  varType: string
}

/** Rust `get_ecl_map_semantics` 的原样返回值 */
export interface EclMapSemanticData {
  sourcePath: string
  version: string
  instructions: EclInstructionSpec[]
  /** ⚠️ 是字符串数组，不是指令对象数组 */
  builtins: string[]
  globals: EclGlobalVar[]
}

/**
 * `loadDefaultEclSemanticData` 的返回值：在后端数据之上补了实际解析到的路径
 * 与失败原因。多份 map 合并时 `resolvedPath` 是用 " + " 连起来的。
 * 全部候选都失败时返回空词表并带 `error`。
 */
export interface LoadedEclSemanticData extends Partial<EclMapSemanticData> {
  version: string
  sourcePath: string
  instructions: EclInstructionSpec[]
  builtins: string[]
  resolvedPath: string
  error?: string
}

/**
 * `normalizeEclSemanticData` 的产物，存进按项目隔离的作用域表。
 * 注意它**不含** `globals`，且 `builtins` 已经并入了全部指令名。
 */
export interface NormalizedEclSemanticData {
  version: string
  sourcePath: string
  instructions: EclInstructionSpec[]
  builtins: string[]
}

/** 对应 `modules/ecl/ai_pack.rs` 的 `AiPackResult`（`rename_all = "camelCase"`） */
export interface AiPackResult {
  skillPath: string
  /** SKILL.md 这次是否被写入（已存在且未 force 时为 false） */
  skillWritten: boolean
  skillExisted: boolean
  /** 每次都会重新生成的 references 文件列表 */
  referenceFiles: string[]
  version: string
}
