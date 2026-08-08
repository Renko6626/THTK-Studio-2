//! MSG 语义数据的加载。
//!
//! 数据是外挂的 `.msgm`（eclmap 家族格式，与 ExpHP/truth 的 `map/` 互换），
//! 版本→表的映射由 `any.msgm` 这个 gamemap 决定。
//!
//! **为什么 th19 / th20 各有一份表**：`thmsg06.c` 的 `th06_find_format()` 用
//! fallthrough 依次查 th19 → th185 → th18 → th16 → ... → th06，新表只增补不
//! 替换。两张表都是按这条链逐条算出来的，所以「表里的 opcode 集合」=「thmsg
//! 解得动的 opcode 集合」，签名来自 thtk 源码这一手数据。
//!
//! th20 **不与 th19 共表**：thtk 把 `case 20:` 与 `case 19:` 并成一个分支，
//! 但那是加新版本时的顺手分组；zero318 逐条整理的 th20 表只到 36，42–56 是
//! th19 作为对战作特有的左右阵营指令。冲突时取窄的一边——详见 `th20.msgm` 抬头。
//!
//! 名字的来源与可信度：opcode 0–36 来自 ExpHP/truth，37 起来自 zero318 并
//! **原样保留** `__` 前缀（他自己的「未坐实」标记）。
//!
//! 未命名的 opcode 会原样显示为 `ins_N`：翻译器双向都对查不到的 opcode 保留
//! 原文，所以**不可能显示错的名字**，只会少几个名字。

use crate::common::{game_map, map_file};
use std::collections::HashMap;

const GAMEMAP: &str = include_str!("../../../assets/maps/any.msgm");
const TH11: &str = include_str!("../../../assets/maps/th11.msgm");
const TH11_ZH: &str = include_str!("../../../assets/maps/th11.msgm.zh.json");
const TH19: &str = include_str!("../../../assets/maps/th19.msgm");
const TH19_ZH: &str = include_str!("../../../assets/maps/th19.msgm.zh.json");
const TH20: &str = include_str!("../../../assets/maps/th20.msgm");
const TH20_ZH: &str = include_str!("../../../assets/maps/th20.msgm.zh.json");

/// 把旁挂的中文说明合并进指令表。
///
/// 中文不进 mapfile 本体——本体要与生态保持一致（只有 `opcode name`），
/// 中文是我们的增量，放在 `<mapfile>.zh.json`。
fn merge_descriptions(data: &mut map_file::MapFileData, zh_json: &str) {
    let map: HashMap<String, String> = match serde_json::from_str(zh_json) {
        Ok(m) => m,
        // 说明缺失不该让整个语义加载失败——没有中文只是少一层注解
        Err(_) => return,
    };
    for ins in &mut data.instructions {
        if let Some(text) = map.get(&ins.opcode.to_string()) {
            ins.description = Some(text.clone());
        }
    }
}

/// 按版本取 MSG 语义数据。**查不到就报错**，不回退到任意一份表。
pub fn parse_msg_semantics(version: &str) -> Result<map_file::MapFileData, String> {
    let id: u32 = version
        .trim()
        .parse()
        .map_err(|_| format!("版本号非法: {version:?}"))?;

    let file = game_map::resolve_map_file(GAMEMAP, id).ok_or_else(|| {
        format!("尚无 th{id} 的 MSG 指令表；本次解包的指令将以 ins_N 原样显示。")
    })?;

    let mut data = match file.as_str() {
        "th11.msgm" => {
            let mut d = map_file::parse_map_content("th11.msgm", TH11)?;
            merge_descriptions(&mut d, TH11_ZH);
            d
        }
        "th19.msgm" => {
            let mut d = map_file::parse_map_content("th19.msgm", TH19)?;
            merge_descriptions(&mut d, TH19_ZH);
            d
        }
        "th20.msgm" => {
            let mut d = map_file::parse_map_content("th20.msgm", TH20)?;
            merge_descriptions(&mut d, TH20_ZH);
            d
        }
        other => return Err(format!("gamemap 指向未内置的表: {other}")),
    };
    data.source_path = file;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn th17_resolves_to_the_shared_table() {
        let data = parse_msg_semantics("17").expect("th17 应有数据");
        assert!(!data.instructions.is_empty());

        let textbox = data
            .instructions
            .iter()
            .find(|i| i.name == "textboxShow")
            .expect("textboxShow 必须在表里");
        assert_eq!(textbox.opcode, 3);
    }

    #[test]
    fn descriptions_are_merged_from_the_sidecar() {
        let data = parse_msg_semantics("17").unwrap();
        let textbox = data.instructions.iter().find(|i| i.opcode == 3).unwrap();
        assert!(
            textbox.description.is_some(),
            "中文说明应从旁挂 .zh.json 合并进来"
        );
    }

    /// th19/th20 的旧指令名字沿用 th11 那份（fallthrough 决定了旧指令在新版本
    /// 里依然有效）。
    #[test]
    fn th19_and_th20_keep_the_legacy_names() {
        for version in ["19", "20"] {
            let data = parse_msg_semantics(version)
                .unwrap_or_else(|e| panic!("th{version} 应有数据: {e}"));

            let textbox = data
                .instructions
                .iter()
                .find(|i| i.opcode == 3)
                .expect("textboxShow 应在表里");
            assert_eq!(textbox.name, "textboxShow");
        }
    }

    /// th19 新增的指令用 zero318 的命名，**原样保留** snake_case 与 `__` 前缀。
    ///
    /// `__` 是 zero318 自己的「未坐实」标记，保留它等于把可信度写进名字本身：
    /// 用户一眼能分辨哪些名字是确认过的（`textboxShow`），哪些还是工作假设
    /// （`__focus_current_side`）。改写成我们的 camelCase 会把这个信号抹掉。
    #[test]
    fn th19_names_new_opcodes_from_zero318_verbatim() {
        let data = parse_msg_semantics("19").unwrap();
        let find = |op: u32| {
            data.instructions
                .iter()
                .find(|i| i.opcode == op)
                .unwrap_or_else(|| panic!("opcode {op} 应在 th19 表里"))
        };

        assert_eq!(find(3).name, "textboxShow", "旧指令仍走 truth 体系");
        assert_eq!(find(37).name, "__initialize_infobox");
        assert_eq!(find(42).name, "__focus_current_side");
        assert_eq!(find(44).name, "__unknown_position_A");
        assert_eq!(find(44).params.len(), 2, "ff = 两个 float");
        assert_eq!(find(56).name, "__gui_unknown_B");
    }

    /// th20 **不与 th19 共表**。
    ///
    /// thtk 的 `th06_find_format()` 把 `case 20:` 和 `case 19:` 并成一个分支，
    /// 但那是 `/* NEWHU: 20 */` 加新版本时的顺手分组，没有验证过——对 thmsg 无害，
    /// 因为 th20 文件里根本不出现那些 opcode。zero318 逐条整理的 th20 表只到 36，
    /// 42–56 那批是 th19（对战作）的左右阵营专用指令，常规 STG 用不上。
    ///
    /// 取窄的一边：th20 真出现 42–56 时我们显示 `ins_N`，而不是显示一个
    /// 可能张冠李戴的名字。
    #[test]
    fn th20_has_its_own_table_without_the_versus_instructions() {
        let data = parse_msg_semantics("20").unwrap();
        assert_eq!(data.source_path, "th20.msgm");
        assert!(
            data.instructions.iter().all(|i| i.opcode <= 36),
            "th20 不该带 th185/th19 才有的 37+ 指令"
        );
        let store = data
            .instructions
            .iter()
            .find(|i| i.opcode == 36)
            .expect("36 应在 th20 表里");
        assert_eq!(store.name, "store");
    }

    /// opcode 26 在 thmsg 的**任何**格式表里都不存在，因此任何版本都不该有它。
    /// 这条钉住的是「表是按 thmsg 的 fallthrough 链算出来的」，不是手抄的。
    #[test]
    fn opcode_26_exists_in_no_version() {
        for version in ["17", "19", "20"] {
            let data = parse_msg_semantics(version).unwrap();
            assert!(
                data.instructions.iter().all(|i| i.opcode != 26),
                "th{version} 不该有 opcode 26"
            );
        }
    }

    /// th17 不该看到 th19 才有的指令。
    #[test]
    fn th17_does_not_leak_th19_instructions() {
        let data = parse_msg_semantics("17").unwrap();
        assert!(
            data.instructions.iter().all(|i| i.opcode != 44),
            "th17 的表不该有 th19 才新增的 44"
        );
    }

    #[test]
    fn covered_versions_all_resolve() {
        for version in [
            "11", "12", "128", "13", "14", "143", "15", "16", "165", "17", "18", "185",
        ] {
            assert!(
                parse_msg_semantics(version).is_ok(),
                "th{version} 应该有数据"
            );
        }
    }

    #[test]
    fn garbage_version_is_rejected() {
        assert!(parse_msg_semantics("abc").is_err());
        assert!(parse_msg_semantics("").is_err());
    }
}
