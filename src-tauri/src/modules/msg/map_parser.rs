//! MSG 语义数据的加载。
//!
//! 数据是外挂的 `.msgm`（eclmap 家族格式，与 ExpHP/truth 的 `map/` 互换），
//! 版本→表的映射由 `any.msgm` 这个 gamemap 决定。
//!
//! **为什么 th19/th20 单独一份表**：`thmsg06.c` 的 `th06_find_format()` 用
//! fallthrough 依次查 th19 → th185 → th18 → th16 → ... → th06，所以 th20 的
//! 指令集是所有旧表的**并集**——新表只增补不替换。`th19.msgm` 因此 = th11 的
//! 全部名字 + th18/th185/th19 三张表的签名（签名抄自 thtk 源码，名字待逆向）。
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

    /// th19/th20 用 th19.msgm：名字来自 th11 那份（fallthrough 决定了旧指令
    /// 在新版本里依然有效），签名另外带上 th18/th185/th19 三张新表。
    #[test]
    fn th19_and_th20_get_legacy_names_plus_new_signatures() {
        for version in ["19", "20"] {
            let data = parse_msg_semantics(version)
                .unwrap_or_else(|e| panic!("th{version} 应有数据: {e}"));

            // 旧指令的名字照常可用
            let textbox = data
                .instructions
                .iter()
                .find(|i| i.opcode == 3)
                .expect("textboxShow 应在表里");
            assert_eq!(textbox.name, "textboxShow");

            // th19 新增的 44 有签名但没名字——宁可显示 ins_44，不编名字
            let new_ins = data
                .instructions
                .iter()
                .find(|i| i.opcode == 44)
                .expect("th19 新增的 44 应被收进来");
            assert_eq!(new_ins.name, "", "没逆向出来就不该有名字");
            assert_eq!(new_ins.params.len(), 2, "ff = 两个 float");
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
