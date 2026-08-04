use anyhow::Result;

use std::fs;
use std::path::Path;

/// 读取工作区里的文本文件。**UTF-8 优先**，不是合法 UTF-8 才按 `fallback_encoding` 解。
///
/// 原实现是无条件 `SHIFT_JIS.decode(&bytes)`，注释却写着"encoding_rs 会自动探测
/// 是否为有效 UTF-8"——它不会。`Encoding::decode` 只认 BOM，无 BOM 就一律按给定
/// 编码解。而 `save_file` 恒以 UTF-8 无 BOM 写盘，于是一写一读不对称：解包出的
/// .dmsg 里的日文对白在编辑器里成了 `蜊夐ｺ鈴怺螟｢`，用 VS Code 打开却是好的。
///
/// 纯 ASCII 两种解法结果相同，所以这个错误长期潜伏——.decl / .dstd 基本只有
/// 指令名和数字，只有 .dmsg 的日文才会触发。
pub fn read_text_file<P: AsRef<Path>>(path: P, fallback_encoding: &str) -> Result<String> {
    let bytes = fs::read(path)?;
    let (text, _used_fallback) =
        crate::common::text_encoding::decode_source_text(&bytes, fallback_encoding);
    Ok(text)
}

/// 写入原始游戏文本文件（Shift-JIS）。装不下的字符**直接失败**。
///
/// 新的 `.decl` / `.dmsg` 一律存 UTF-8，只有 `.msg` / `.txt` 这种直接喂给游戏的
/// 原始文件才走这里。
///
/// 原实现是 `SHIFT_JIS.encode()` + `println!` 警告 + 照常写盘——三处都错：
/// encode 对装不下的字符会写入 HTML 数字实体（`"你"` → `"&#20320;"`），警告打到
/// stdout 而 Windows 上的 GUI 应用没有控制台（`cmd_runner` 专门隐藏了它），
/// 于是用户拿到一个看似成功、实则内容损坏的文件。
pub fn write_file_sjis<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let bytes = crate::common::text_encoding::encode_strict(content, "shift-jis").map_err(
        |offenders| {
            anyhow::anyhow!(crate::common::text_encoding::describe_unmappable(
                &offenders,
                "shift-jis"
            ))
        },
    )?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn write_file_utf8<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}
