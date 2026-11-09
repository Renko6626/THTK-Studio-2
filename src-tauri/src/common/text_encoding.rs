//! 工作区文本读取与 ECL 文件写入所用的编码处理。
//! MSG/END 二进制字符串编码由随包 truth 承担。

use encoding_rs::{Encoding, GBK, SHIFT_JIS, UTF_8};

/// 允许的编码名。与 `.thtk-project.json` 的 `encoding` 字段取值一致。
pub const SUPPORTED_ENCODINGS: [&str; 3] = ["shift-jis", "gbk", "utf-8"];

/// 编码名 → encoding_rs 编码器。大小写不敏感，允许 `shift_jis` / `sjis` 等常见写法。
pub fn resolve(name: &str) -> Result<&'static Encoding, String> {
    match name.trim().to_lowercase().replace('_', "-").as_str() {
        "shift-jis" | "shiftjis" | "sjis" | "cp932" => Ok(SHIFT_JIS),
        "gbk" | "gb2312" | "cp936" => Ok(GBK),
        "utf-8" | "utf8" => Ok(UTF_8),
        other => Err(format!(
            "不支持的编码: {other:?}（可用 {}）",
            SUPPORTED_ENCODINGS.join(" / ")
        )),
    }
}

/// 一个装不下的字符及其位置，用于把错误指到源文件的具体位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmappableChar {
    /// 1 起的行号
    pub line: usize,
    /// 1 起的列号（按字符计，不是字节）
    pub column: usize,
    pub ch: char,
}

/// 编码文本，目标编码装不下任何字符时**失败**而不是写入 HTML 实体。
///
/// 逐字符试编码来定位问题字符。Shift-JIS / GBK 都是无状态编码，单独编码与
/// 在上下文中编码结果一致，所以这样定位是准确的。.msg 文本只有几 KB，开销可忽略。
pub fn encode_strict(text: &str, encoding_name: &str) -> Result<Vec<u8>, Vec<UnmappableChar>> {
    let encoding = match resolve(encoding_name) {
        Ok(encoding) => encoding,
        // 编码名非法时不该走到这里（调用方应先 resolve），保守起见按"全部装不下"处理
        Err(_) => return Err(Vec::new()),
    };

    let mut offenders = Vec::new();
    for (line_index, line) in text.split('\n').enumerate() {
        for (char_index, ch) in line.chars().enumerate() {
            let mut buffer = [0u8; 4];
            let (_, _, had_unmappable) = encoding.encode(ch.encode_utf8(&mut buffer));
            if had_unmappable {
                offenders.push(UnmappableChar {
                    line: line_index + 1,
                    column: char_index + 1,
                    ch,
                });
            }
        }
    }

    if !offenders.is_empty() {
        return Err(offenders);
    }

    let (bytes, _, had_unmappable) = encoding.encode(text);
    if had_unmappable {
        // 逐字符检查没抓到却整体报了——不该发生，但绝不能把带实体的产物写出去
        return Err(Vec::new());
    }
    Ok(bytes.into_owned())
}

/// 把 [`encode_strict`] 的失败列表转成给用户看的一句话。
///
/// 只列前几个并给出改用 GBK 的建议——汉化场景下这几乎总是正确的下一步。
pub fn describe_unmappable(offenders: &[UnmappableChar], encoding_name: &str) -> String {
    if offenders.is_empty() {
        return format!("有字符无法用 {encoding_name} 编码，但未能定位到具体位置。");
    }

    const MAX_LISTED: usize = 5;
    let listed = offenders
        .iter()
        .take(MAX_LISTED)
        .map(|o| format!("第 {} 行第 {} 列的 {:?}", o.line, o.column, o.ch))
        .collect::<Vec<_>>()
        .join("、");
    let more = if offenders.len() > MAX_LISTED {
        format!("，另有 {} 处", offenders.len() - MAX_LISTED)
    } else {
        String::new()
    };

    let hint = if encoding_name == "gbk" {
        "GBK 已经能同时容纳简体汉字与日文，出现这种情况通常是文本里混进了别的语言或特殊符号。"
    } else {
        "东方汉化版一般用 GBK——它同时装得下简体汉字和日文假名/汉字。可在项目设置里改，或在本次操作的编码下拉里临时切换。"
    };

    format!("{listed}{more} 无法用 {encoding_name} 编码。{hint}")
}

/// 读取工作区里的文本文件：**UTF-8 优先**，不是合法 UTF-8 才按 `fallback` 解。
///
/// 返回 `(文本, 是否走了 fallback)`。
///
/// 顺序不能反。THTK-Studio 自己产出的 `.decl` / `.msgspec` / `.stdspec` 一律以 UTF-8
/// 无 BOM 写盘（`save_file` 恒传 `is_source = true`），若读取时无条件按 Shift-JIS
/// 解，日文对白会变成 `蜊夐ｺ鈴怺螟｢` 这样的乱码——而纯 ASCII 的文件往返完全正常，
/// 所以这个错误能长期潜伏：`.decl` / `.stdspec` 基本只有指令名和数字，只有 `.msgspec`
/// 里的日文才触发。
///
/// fallback 留给用户直接打开的外部文件（原始游戏文本等），取项目配置的编码。
pub fn decode_source_text(bytes: &[u8], fallback_encoding: &str) -> (String, bool) {
    // 去掉 UTF-8 BOM，否则它会变成正文开头的 U+FEFF
    let body = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);

    if let Ok(text) = std::str::from_utf8(body) {
        return (text.to_string(), false);
    }

    let encoding = resolve(fallback_encoding).unwrap_or(SHIFT_JIS);
    let (text, _, _) = encoding.decode(body);
    (text.into_owned(), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_common_spellings() {
        assert_eq!(resolve("shift-jis").unwrap().name(), SHIFT_JIS.name());
        assert_eq!(resolve("Shift_JIS").unwrap().name(), SHIFT_JIS.name());
        assert_eq!(resolve("  SJIS ").unwrap().name(), SHIFT_JIS.name());
        assert_eq!(resolve("gbk").unwrap().name(), GBK.name());
        assert_eq!(resolve("utf-8").unwrap().name(), UTF_8.name());
        assert!(resolve("big5").is_err());
    }

    /// 核心回归：原实现用 `SHIFT_JIS.encode` 且丢掉了第三个返回值，
    /// 简体汉字会静默变成 `&#20320;` 这样的 HTML 实体写进 .msg。
    #[test]
    fn simplified_chinese_fails_under_shift_jis_instead_of_emitting_entities() {
        let result = encode_strict("你好世界", "shift-jis");
        let offenders = result.expect_err("简体汉字不该能编成 Shift-JIS");
        assert!(!offenders.is_empty(), "必须能定位到具体字符");
        assert!(offenders.iter().any(|o| o.ch == '你'));
    }

    #[test]
    fn never_returns_bytes_containing_html_entities() {
        // 直接用 encoding_rs 会得到实体；encode_strict 必须是 Err
        let (raw, _, _) = SHIFT_JIS.encode("你好");
        assert!(
            String::from_utf8_lossy(&raw).contains("&#"),
            "前提：encoding_rs 确实会写实体，这条测试才有意义"
        );
        assert!(encode_strict("你好", "shift-jis").is_err());
    }

    #[test]
    fn japanese_encodes_fine_under_shift_jis() {
        let bytes = encode_strict("博麗霊夢だぜ", "shift-jis").expect("日文应能编码");
        assert!(!bytes.is_empty());
        assert!(!String::from_utf8_lossy(&bytes).contains("&#"));
    }

    /// GBK 是汉化版的答案：简日都装得下。
    #[test]
    fn gbk_holds_both_simplified_chinese_and_japanese() {
        assert!(encode_strict("你好，博丽灵梦", "gbk").is_ok());
        assert!(encode_strict("博麗霊夢だぜ", "gbk").is_ok());
    }

    #[test]
    fn utf8_holds_everything() {
        assert!(encode_strict("你好 博麗霊夢 ①②③", "utf-8").is_ok());
    }

    #[test]
    fn locates_offenders_by_line_and_column() {
        let offenders = encode_strict("あいう\nかき你", "shift-jis").unwrap_err();
        assert_eq!(offenders.len(), 1);
        assert_eq!(offenders[0].line, 2);
        assert_eq!(offenders[0].column, 3);
        assert_eq!(offenders[0].ch, '你');
    }

    #[test]
    fn describes_offenders_with_position_and_suggestion() {
        let offenders = encode_strict("あ你", "shift-jis").unwrap_err();
        let message = describe_unmappable(&offenders, "shift-jis");
        assert!(message.contains("第 1 行第 2 列"));
        assert!(message.contains("GBK"), "应建议改用 GBK: {message}");
    }

    /// ★关键的领域事实：简体汉字**大部分能"成功"编进 Shift-JIS**。
    ///
    /// `"你好世界再见朋友"` 八个字里只有「你」「见」装不下——「好世界再朋友」
    /// 这些字形日文汉字里也有。所以用 SJIS 打包中文不会整体失败，而是少数字
    /// 变成实体、其余静默变成日文字形。这正是"大部分正常、个别地方冒怪东西"
    /// 这种迷惑症状的由来，也是本模块必须逐字符检查而不能只看整体返回值的原因。
    #[test]
    fn simplified_chinese_only_partially_fails_under_shift_jis() {
        let offenders = encode_strict("你好世界再见朋友", "shift-jis").unwrap_err();
        let bad: Vec<char> = offenders.iter().map(|o| o.ch).collect();
        assert_eq!(bad, vec!['你', '见'], "只有简体独有字形装不下");
    }

    #[test]
    fn describe_lists_at_most_five_then_summarizes() {
        // 全部选简体独有字形，确保超过 5 个装不下
        let offenders = encode_strict("你见丽灵这简说汉", "shift-jis").unwrap_err();
        assert!(
            offenders.len() > 5,
            "夹具应全是装不下的字，实际只有 {} 个：{:?}",
            offenders.len(),
            offenders.iter().map(|o| o.ch).collect::<Vec<_>>()
        );
        let message = describe_unmappable(&offenders, "shift-jis");
        assert!(
            message.contains("另有"),
            "超过 5 个应汇总剩余数量: {message}"
        );
    }

    // ---- decode_source_text：读工作区文本文件 ----

    /// 回归：解包产出的 .msgspec 是 UTF-8 无 BOM，此前读取时无条件按 Shift-JIS 解，
    /// 日文对白在编辑器里变成 `蜊夐ｺ鈴怺螟｢...`，而 VS Code 打开却是好的。
    #[test]
    fn utf8_japanese_survives_the_round_trip() {
        let original = "博麗霊夢「そこまでよ！」";
        let (text, used_fallback) = decode_source_text(original.as_bytes(), "shift-jis");
        assert_eq!(text, original);
        assert!(!used_fallback, "合法 UTF-8 不该走 fallback");
    }

    /// 纯 ASCII 两种解法结果相同——这正是该 bug 能长期潜伏的原因：
    /// .decl / .stdspec 基本只有指令名和数字，看不出问题。
    #[test]
    fn ascii_is_identical_either_way() {
        let source = b"entry 0\n    textboxShow()\n";
        let (text, used_fallback) = decode_source_text(source, "shift-jis");
        assert_eq!(text, "entry 0\n    textboxShow()\n");
        assert!(!used_fallback);
    }

    #[test]
    fn falls_back_for_genuine_shift_jis_files() {
        let sjis = encode_strict("博麗霊夢", "shift-jis").unwrap();
        let (text, used_fallback) = decode_source_text(&sjis, "shift-jis");
        assert_eq!(text, "博麗霊夢");
        assert!(used_fallback, "非 UTF-8 才该走 fallback");
    }

    #[test]
    fn falls_back_to_the_configured_encoding_not_always_shift_jis() {
        let gbk = encode_strict("博丽灵梦，你好世界", "gbk").unwrap();
        let (text, used_fallback) = decode_source_text(&gbk, "gbk");
        assert_eq!(text, "博丽灵梦，你好世界");
        assert!(used_fallback);
    }

    #[test]
    fn strips_the_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("あいう".as_bytes());
        let (text, _) = decode_source_text(&bytes, "shift-jis");
        assert_eq!(text, "あいう", "BOM 不该混进正文");
    }

    #[test]
    fn empty_file_is_not_an_error() {
        assert_eq!(decode_source_text(b"", "shift-jis"), (String::new(), false));
    }
}
