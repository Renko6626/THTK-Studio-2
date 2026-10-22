//! MSG 语义数据的加载。
//!
//! 数据是外挂的 `.msgm` / `.endm`（eclmap 家族格式，与 ExpHP/truth 的 `map/`
//! 互换），版本→表的映射由同名 gamemap 决定。加载后再套一层
//! [`signatures`] 的逐版本档案——`thmsg` 是按版本 fallthrough 的，单张表
//! 无法表达 TH14/TH16/TH18 的参数变化。
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

use super::signatures;
use super::MsgScriptKind;

const GAMEMAP_STAGE: &str = include_str!("../../../assets/maps/any.msgm");
const GAMEMAP_ENDING: &str = include_str!("../../../assets/maps/any.endm");

const TH10: &str = include_str!("../../../assets/maps/th10.msgm");
const TH11: &str = include_str!("../../../assets/maps/th11.msgm");
const TH11_ZH: &str = include_str!("../../../assets/maps/th11.msgm.zh.json");
const TH19: &str = include_str!("../../../assets/maps/th19.msgm");
const TH19_ZH: &str = include_str!("../../../assets/maps/th19.msgm.zh.json");
const TH20: &str = include_str!("../../../assets/maps/th20.msgm");
const TH20_ZH: &str = include_str!("../../../assets/maps/th20.msgm.zh.json");

const TH10_END: &str = include_str!("../../../assets/maps/th10.endm");
const TH10_END_ZH: &str = include_str!("../../../assets/maps/th10.endm.zh.json");
const TH20_END: &str = include_str!("../../../assets/maps/th20.endm");
const TH20_END_ZH: &str = include_str!("../../../assets/maps/th20.endm.zh.json");

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

fn parse_version(version: &str) -> Result<u32, String> {
    version
        .trim()
        .parse()
        .map_err(|_| format!("版本号非法: {version:?}"))
}

/// 关卡对话（stage MSG）语义数据。
pub fn parse_stage_semantics(version: &str) -> Result<map_file::MapFileData, String> {
    let id = parse_version(version)?;
    let file = game_map::resolve_map_file(GAMEMAP_STAGE, id).ok_or_else(|| {
        format!("尚无 th{id} 的 MSG 指令表；本次解包的指令将以 ins_N 原样显示。")
    })?;

    let mut data = match file.as_str() {
        "th10.msgm" => map_file::parse_map_content("th10.msgm", TH10)?,
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

    signatures::apply_stage_profile(&mut data, id);
    data.source_path = file;
    Ok(data)
}

/// 结局 / staff（`thmsg -e`）语义数据。
///
/// thtk 的 `-e` 只支持 th10+；更早的作品是另一套格式，这里明确报错，
/// 不拿 stage 表或 th10 表去冒充。
pub fn parse_ending_semantics(version: &str) -> Result<map_file::MapFileData, String> {
    let id = parse_version(version)?;
    if id < 10 {
        return Err(format!(
            "thmsg 的结局/staff 模式（-e）从 th10 起支持；th{id} 没有对应的表。"
        ));
    }

    let file = game_map::resolve_map_file(GAMEMAP_ENDING, id).ok_or_else(|| {
        format!("尚无 th{id} 的结局/staff 指令表；本次解包的指令将以 ins_N 原样显示。")
    })?;

    let mut data = match file.as_str() {
        "th10.endm" => {
            let mut d = map_file::parse_map_content("th10.endm", TH10_END)?;
            merge_descriptions(&mut d, TH10_END_ZH);
            d
        }
        "th20.endm" => {
            let mut d = map_file::parse_map_content("th20.endm", TH20_END)?;
            merge_descriptions(&mut d, TH20_END_ZH);
            d
        }
        other => return Err(format!("结局 gamemap 指向未内置的表: {other}")),
    };

    signatures::apply_ending_profile(&mut data, id);
    data.source_path = file;
    Ok(data)
}

/// 按脚本类型取语义数据。**查不到就报错**，不回退到另一种类型的表。
pub fn parse_msg_semantics(
    version: &str,
    kind: MsgScriptKind,
) -> Result<map_file::MapFileData, String> {
    match kind {
        MsgScriptKind::Stage => parse_stage_semantics(version),
        MsgScriptKind::Ending => parse_ending_semantics(version),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(version: &str) -> map_file::MapFileData {
        parse_stage_semantics(version).expect("stage semantics")
    }

    fn ending(version: &str) -> map_file::MapFileData {
        parse_ending_semantics(version).expect("ending semantics")
    }

    fn by_opcode(data: &map_file::MapFileData, opcode: u32) -> &map_file::MapInstruction {
        data.instructions
            .iter()
            .find(|i| i.opcode == opcode)
            .unwrap_or_else(|| panic!("opcode {opcode} 应在表里"))
    }

    #[test]
    fn th17_resolves_to_the_shared_table() {
        let data = stage("17");
        assert!(!data.instructions.is_empty());
        assert_eq!(by_opcode(&data, 3).name, "textboxShow");
    }

    #[test]
    fn descriptions_are_merged_from_the_sidecar() {
        let data = stage("17");
        assert!(by_opcode(&data, 3).description.is_some());
    }

    #[test]
    fn th10_uses_its_own_shifted_names() {
        let data = stage("10");
        assert_eq!(data.source_path, "th10.msgm");
        assert_eq!(by_opcode(&data, 9).name, "skippable");
        assert_eq!(by_opcode(&data, 14).name, "textLine1");
        assert_eq!(by_opcode(&data, 16).name, "textAdd");
        assert_eq!(by_opcode(&data, 17).name, "textClear");
        // TH10 还没有 focus-none。op25 在 thtk 表里有签名但没有名字，
        // 保留为可编译的未命名条目。
        assert!(data.instructions.iter().all(|i| i.opcode <= 25));
        assert!(by_opcode(&data, 25).name.is_empty());
        assert_eq!(by_opcode(&data, 25).params.len(), 1);
    }

    #[test]
    fn later_stage_names_are_filtered_by_version() {
        // TH11 到 26 为止；27 的 musicFade 是 TH12 才有的。
        let th11 = stage("11");
        assert!(!th11.instructions.iter().any(|i| i.opcode == 27));
        assert!(!th11.instructions.iter().any(|i| i.opcode == 36));

        // TH13 新增 31，TH14 新增 32；更早的版本不该看到它们。
        assert_eq!(by_opcode(&stage("13"), 31).name, "showEnemy1");
        assert_eq!(by_opcode(&stage("14"), 32).name, "modern32");
        assert!(!stage("12").instructions.iter().any(|i| i.opcode == 31));
        assert!(!stage("13").instructions.iter().any(|i| i.opcode == 32));

        // TH16 有 33–35，没有 TH18 的 36。
        let th16 = stage("16");
        assert_eq!(by_opcode(&th16, 33).name, "portraitDarken");
        assert_eq!(by_opcode(&th16, 35).name, "lightsOut");
        assert!(!th16.instructions.iter().any(|i| i.opcode == 36));

        // TH18 多出 36 store。
        assert_eq!(by_opcode(&stage("18"), 36).name, "store");
    }

    #[test]
    fn th143_and_th165_use_tutorial_for_opcode_33() {
        for version in ["143", "165"] {
            let data = stage(version);
            assert_eq!(by_opcode(&data, 33).name, "tutorial", "th{version}");
            assert!(!data.instructions.iter().any(|i| i.opcode == 34));
        }
    }

    #[test]
    fn th185_adds_bm_info_names_but_not_the_thtk_gap_opcodes() {
        let data = stage("185");
        assert_eq!(by_opcode(&data, 37).name, "bmInfoOpen");
        assert_eq!(by_opcode(&data, 38).name, "bmFocusInfo");
        assert_eq!(by_opcode(&data, 39).name, "bmInfoClose");
        // 40/41 在 thpages 有名字，但 thtk 没有格式表项；不命名，避免
        // 生成 thmsg 无法编译的 .dmsg。
        assert!(!data.instructions.iter().any(|i| i.opcode == 40));
        assert!(!data.instructions.iter().any(|i| i.opcode == 41));
    }

    /// th19/th20 的旧指令名字沿用 th11 那份（fallthrough 决定了旧指令在新版本
    /// 里依然有效）。
    #[test]
    fn th19_and_th20_keep_the_legacy_names() {
        for version in ["19", "20"] {
            let data = stage(version);
            assert_eq!(by_opcode(&data, 3).name, "textboxShow");
        }
    }

    /// th19 新增的指令用 zero318 的命名，**原样保留** snake_case 与 `__` 前缀。
    #[test]
    fn th19_names_new_opcodes_from_zero318_verbatim() {
        let data = stage("19");
        assert_eq!(by_opcode(&data, 37).name, "__initialize_infobox");
        assert_eq!(by_opcode(&data, 42).name, "__focus_current_side");
        assert_eq!(by_opcode(&data, 44).name, "__unknown_position_A");
        assert_eq!(by_opcode(&data, 44).params.len(), 2, "ff = 两个 float");
        assert_eq!(by_opcode(&data, 56).name, "__gui_unknown_B");
    }

    #[test]
    fn th20_has_its_own_names_but_keeps_decodable_opcodes_unnamed() {
        let data = stage("20");
        assert_eq!(data.source_path, "th20.msgm");
        assert_eq!(by_opcode(&data, 36).name, "store");

        // th19 的对战作指令不借名字，但仍保留可解码条目：显示 ins_N、
        // 参数形状可校验、打包不被拒绝。
        for opcode in [37, 38, 39, 42, 44, 46, 56] {
            assert!(
                by_opcode(&data, opcode).name.is_empty(),
                "opcode {opcode} 在 th20 里不应有名字"
            );
        }
        assert_eq!(by_opcode(&data, 44).params.len(), 2);
        assert!(!data
            .instructions
            .iter()
            .any(|i| i.name == "__focus_current_side"));
    }

    #[test]
    fn opcode_26_exists_in_no_version() {
        for version in ["17", "19", "20"] {
            let data = stage(version);
            assert!(data.instructions.iter().all(|i| i.opcode != 26));
        }
    }

    #[test]
    fn th17_does_not_leak_th19_instructions() {
        let data = stage("17");
        assert!(data.instructions.iter().all(|i| i.opcode != 44));
    }

    /// 关键的版本精确性：同一个 opcode 在不同版本上的参数形状必须不同。
    #[test]
    fn stage_signatures_are_version_accurate() {
        assert_eq!(by_opcode(&stage("13"), 13).params.len(), 1);
        assert_eq!(by_opcode(&stage("18"), 13).params.len(), 2);
        assert_eq!(by_opcode(&stage("18"), 4).params.len(), 1);
        assert_eq!(by_opcode(&stage("16"), 4).params.len(), 0);
        assert_eq!(by_opcode(&stage("14"), 5).params.len(), 1);
        assert_eq!(by_opcode(&stage("13"), 5).params.len(), 0);
        assert_eq!(by_opcode(&stage("185"), 19).params.len(), 1);
        // TH16–18 的真实 musicBoss 是零参数（TH185 才加曲目索引）。
        assert_eq!(by_opcode(&stage("16"), 19).params.len(), 0);
        assert_eq!(by_opcode(&stage("18"), 19).params.len(), 0);
        assert_eq!(by_opcode(&stage("19"), 44).params.len(), 2);
        // th20 保留未命名条目，所以参数形状仍然可用。
        assert_eq!(by_opcode(&stage("20"), 44).params.len(), 2);
    }

    #[test]
    fn ending_tables_are_separate_from_stage_tables() {
        let end = ending("18");
        assert_eq!(end.source_path, "th10.endm");
        assert_eq!(by_opcode(&end, 3).name, "textAdd");
        assert_eq!(by_opcode(&end, 7).name, "anmSourceLoad");
        assert_eq!(by_opcode(&end, 7).signature.as_deref(), Some("Sz"));
        // 结局表里没有 stage 的 playerShow / textboxShow。
        assert!(end.instructions.iter().all(|i| i.opcode != 1));
        assert!(end.instructions.iter().all(|i| i.name != "textboxShow"));
        // op4/op13 在引擎里有，但 thtk 结局表没有，故意不命名。
        assert!(end.instructions.iter().all(|i| i.opcode != 4));
        assert!(end.instructions.iter().all(|i| i.opcode != 13));
    }

    #[test]
    fn th20_ending_overrides_text_color_to_two_ints() {
        let end = ending("20");
        assert_eq!(end.source_path, "th20.endm");
        assert_eq!(by_opcode(&end, 9).params.len(), 2);
        // 未覆盖的条目继续走 th10 表。
        assert_eq!(by_opcode(&end, 7).signature.as_deref(), Some("Sz"));
    }

    #[test]
    fn ending_mode_rejects_pre_th10() {
        assert!(parse_ending_semantics("9").is_err());
        assert!(parse_ending_semantics("6").is_err());
    }

    #[test]
    fn covered_versions_all_resolve() {
        for version in [
            "10", "11", "12", "128", "13", "14", "143", "15", "16", "165", "17", "18", "185",
            "19", "20",
        ] {
            assert!(
                parse_stage_semantics(version).is_ok(),
                "th{version} 应该有 stage 数据"
            );
        }
        for version in [
            "10", "11", "12", "128", "13", "14", "143", "15", "16", "165", "17", "18", "185",
            "19", "20",
        ] {
            assert!(
                parse_ending_semantics(version).is_ok(),
                "th{version} 应该有 ending 数据"
            );
        }
    }

    #[test]
    fn garbage_version_is_rejected() {
        assert!(parse_stage_semantics("abc").is_err());
        assert!(parse_stage_semantics("").is_err());
    }
}
