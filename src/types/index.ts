/**
 * 前后端契约的统一出口。
 *
 * 这些类型逐字段对照 `src-tauri/src/` 的结构体写成，改动前先确认 Rust 侧也改了。
 * 注意后端的序列化命名风格并不统一（camelCase / snake_case / 个别字段单独 rename），
 * 各文件的注释里标了来源与坑位。
 */
export * from './ecl'
export * from './fs'
export * from './project'
export * from './toolchain'
