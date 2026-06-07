use anyhow::Result;
use encoding_rs::{SHIFT_JIS, UTF_8};
use std::fs;
use std::path::Path;

// 读取文件：二进制 -> 猜测编码 -> UTF-8 字符串
pub fn read_text_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let bytes = fs::read(path)?;
    // encoding_rs 会自动探测是否为有效 UTF-8，如果不是则尝试用 Shift-JIS 解码
    // .0 是解码后的字符串 (Cow), .1 是使用的编码, .2 是是否有错误
    let (cow, _encoding, _had_errors) = SHIFT_JIS.decode(&bytes);
    Ok(cow.into_owned())
}

// 写入文件：UTF-8 字符串 -> Shift-JIS 二进制
// 注意：新的 .decl / .dmsg 建议直接保存为 UTF-8。只有 .msg/.txt 这种原始文件需要转码。
// 这里我们提供两个函数供选择。
pub fn write_file_sjis<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let (cow, _, unmappable) = SHIFT_JIS.encode(content);
    if unmappable {
        // 这里可以在未来加入 warning 日志
        println!("Warning: Some characters could not be mapped to Shift-JIS");
    }
    fs::write(path, cow)?;
    Ok(())
}

pub fn write_file_utf8<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}
