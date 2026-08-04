//! 游戏文本的编码处理。
//!
//! 东方原作的 .msg 是 Shift-JIS；汉化版通常改用 GBK（它同时装得下简体汉字和
//! 日文假名/汉字，而汉化版常有简日混排）；少数走 thcrap 一类方案的用 UTF-8。
//!
//! 这里存在的理由是 `encoding_rs` 的两个**静默**行为，直接用会产出坏数据：
//!
//! 1. `Encoding::encode` 遇到目标编码装不下的字符时**不报错**，而是写入 HTML
//!    数字实体。`SHIFT_JIS.encode("你好")` 得到的是字面的 `&#20320;`，打包"成功"，
//!    游戏里显示的却是这串实体本身。更隐蔽的是部分字符会被映射成别的日文汉字，
//!    连实体都不留，看上去是通顺的错字。
//! 2. `Encoding::decode` 用 Shift-JIS 解几乎任何字节序列都能"成功"——它的单字节
//!    区覆盖极广，所以 `had_errors` 恒为 false。把 GBK 的 .msg 当 SJIS 解出来
//!    满屏乱码，却没有任何自动信号。
//!
//! 因此本模块只提供两个**会说话**的入口：[`encode_strict`] 装不下就失败并指出
//! 是哪些字符，[`decode_with_warning`] 解出可疑结果时附一条提示。

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

/// 写入游戏文件前对目标编码本身的告诫，没有就返回 None。
///
/// 依据：thcrap 官方文档明说「the original games **don't support any form of
/// Unicode**」「the game uses a NUL byte as a string delimiter (because it works
/// with plain C strings in **SHIFT-JIS** encoding)」——thcrap 能做多语言是靠运行时
/// 把游戏进程里的 Win32 `A` 函数换成 `U` 版本（`TextOutExA` → `TextOutExU`），
/// **它根本不改 .msg 文件**。所以往 .msg 里塞 UTF-8 字节，原版游戏必然读不了。
///
/// GBK 则是"有条件可用"：传统汉化走改 `CreateFontIndirectA` 的 `lfCharSet`
/// （`SHIFTJIS_CHARSET` 0x80 → `GB2312_CHARSET` 0x86）加转区那条路，还得处理
/// 字节边界判断。能不能跑通取决于用户的补丁环境，不是本工具能保证的。
pub fn caution_for_game_file(encoding_name: &str) -> Option<String> {
    match resolve(encoding_name).ok()?.name() {
        n if n == UTF_8.name() => Some(
            "原版东方游戏不支持任何形式的 Unicode（按 Shift-JIS 的 C 字符串处理），\
             UTF-8 的 .msg 在原版游戏里读不出来。仅在你确知目标引擎能读 UTF-8 时使用。"
                .to_string(),
        ),
        n if n == GBK.name() => Some(
            "GBK 需要游戏侧已做汉化适配（字体 charset 补丁、字节边界判断、转区等）。\
             原版未打补丁的游戏读 GBK 文本会乱码。"
                .to_string(),
        ),
        _ => None,
    }
}

/// 解码字节，并在结果疑似乱码时附一条提示。
///
/// 返回 `(文本, 可疑提示)`。Shift-JIS 解什么都不报错，所以不能依赖 `had_errors`，
/// 只能看解出来的字符长什么样。
pub fn decode_with_warning(bytes: &[u8], encoding_name: &str) -> (String, Option<String>) {
    let encoding = match resolve(encoding_name) {
        Ok(encoding) => encoding,
        Err(_) => SHIFT_JIS,
    };
    let (text, _, _) = encoding.decode(bytes);
    let text = text.into_owned();
    let warning = detect_mojibake(&text).map(|reason| {
        format!("{reason}当前按 {encoding_name} 解包；若内容不对，换个编码重新解包即可（原文件不会被改动）。")
    });
    (text, warning)
}

/// 乱码启发式。
///
/// 用错编码解 Shift-JIS 时的典型特征是**大量半角片假名**（GBK/UTF-8 的高位字节
/// 落进 SJIS 的 0xA1–0xDF 单字节半角区），其次是私用区字符和替换字符。
///
/// 阈值取得保守：正常的日文 .msg 偶尔也会用半角片假名，宁可漏报也不要在正确
/// 解包的文件上天天弹提示。返回 None 表示没有明显嫌疑，**不代表一定正确**。
pub fn detect_mojibake(text: &str) -> Option<String> {
    let total = text.chars().count();
    if total < 16 {
        return None;
    }

    let suspicious = text
        .chars()
        .filter(|&c| {
            matches!(c,
                '\u{FF61}'..='\u{FF9F}'   // 半角片假名
                | '\u{E000}'..='\u{F8FF}' // 私用区
                | '\u{FFFD}'              // 替换字符
            )
        })
        .count();

    let ratio = suspicious as f64 / total as f64;
    if suspicious >= 8 && ratio > 0.15 {
        Some(format!(
            "解包结果里有 {suspicious} 个可疑字符（占 {:.0}%），像是编码不匹配。",
            ratio * 100.0
        ))
    } else {
        None
    }
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
        assert!(message.contains("另有"), "超过 5 个应汇总剩余数量: {message}");
    }

    /// 核心回归：用错编码解包时几乎没有可靠的自动信号，只能靠字符特征。
    ///
    /// `decode` 的 `had_errors` 不能用——Shift-JIS 单字节区覆盖极广，GBK 字节
    /// 绝大多数都能"解出"东西；只有末尾字节恰好不成对时才会冒一个替换字符。
    /// 靠它判断等于把检出率押在文本长度的奇偶性上。
    #[test]
    fn flags_gbk_bytes_decoded_as_shift_jis() {
        let gbk_bytes = encode_strict(
            "你好世界，这是一段足够长的简体中文测试文本用来触发启发式检测",
            "gbk",
        )
        .unwrap();

        let (decoded, _, _) = SHIFT_JIS.decode(&gbk_bytes);
        let replacement_chars = decoded.chars().filter(|&c| c == '\u{FFFD}').count();
        assert!(
            replacement_chars <= 2,
            "前提：整段乱码里替换字符寥寥无几（实际 {replacement_chars} 个），\
             所以 had_errors 不足以作为信号，必须看字符分布"
        );

        let (_, warning) = decode_with_warning(&gbk_bytes, "shift-jis");
        assert!(warning.is_some(), "应当提示疑似编码不匹配");
        assert!(warning.unwrap().contains("编码"));
    }

    #[test]
    fn does_not_flag_correctly_decoded_japanese() {
        let sjis = encode_strict(
            "博麗霊夢「そこまでよ！\n魔理沙、あなたの負けね」\nこれは正しい日本語のテキストです。",
            "shift-jis",
        )
        .unwrap();
        let (text, warning) = decode_with_warning(&sjis, "shift-jis");
        assert!(text.contains("博麗霊夢"));
        assert_eq!(warning, None, "正确解包不该弹提示");
    }

    #[test]
    fn does_not_flag_correctly_decoded_chinese() {
        let gbk = encode_strict(
            "博丽灵梦「到此为止了！\n魔理沙，你输了」\n这是一段正常的简体中文文本。",
            "gbk",
        )
        .unwrap();
        let (text, warning) = decode_with_warning(&gbk, "gbk");
        assert!(text.contains("博丽灵梦"));
        assert_eq!(warning, None);
    }

    #[test]
    fn short_text_is_never_flagged() {
        assert_eq!(detect_mojibake("ﾃｽﾄ"), None, "太短的文本不做判断");
    }

    /// 原版游戏读不了 UTF-8（thcrap 文档：games don't support any form of Unicode），
    /// 允许选但必须说出来，否则用户会打出一批在原版里全是乱码的 .msg。
    #[test]
    fn warns_that_utf8_is_unreadable_by_original_games() {
        let caution = caution_for_game_file("utf-8").expect("UTF-8 必须给告诫");
        assert!(caution.contains("Unicode"));
        assert!(caution.contains("读不出来") || caution.contains("读不了"));
    }

    #[test]
    fn warns_that_gbk_needs_a_patched_game() {
        let caution = caution_for_game_file("gbk").expect("GBK 必须说明前提");
        assert!(caution.contains("汉化适配") || caution.contains("补丁"));
    }

    #[test]
    fn shift_jis_needs_no_caution() {
        assert_eq!(caution_for_game_file("shift-jis"), None, "原生编码不该弹告诫");
    }
}
