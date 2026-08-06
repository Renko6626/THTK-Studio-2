//! MSG 语义数据的加载。
//!
//! 数据是外挂的 `.msgm`（eclmap 家族格式，与 ExpHP/truth 的 `map/` 互换），
//! 版本→表的映射由 `any.msgm` 这个 gamemap 决定。
//!
//! **为什么不是"所有版本共用一份表"**：`thmsg.c` 的 `th06_find_format()` 是
//! 级联穿透的，签名表按版本组分开——`th19_msg_fmts[]` 给 th19/th20 新增了
//! 42–47、50–56 共 13 条，`th185_msg_fmts[]` 又有 37/38/39。所以 th19/th20
//! **不在** gamemap 里，缺表时如实报错而不是拿 th17 的表顶上。

use crate::common::{game_map, map_file};
use std::collections::HashMap;

const GAMEMAP: &str = include_str!("../../../assets/maps/any.msgm");
const TH11: &str = include_str!("../../../assets/maps/th11.msgm");
const TH11_ZH: &str = include_str!("../../../assets/maps/th11.msgm.zh.json");

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
        format!(
            "尚无 th{id} 的 MSG 指令表。thmsg 对 th19/th20 有独立的签名表\
             （th19_msg_fmts，新增 13 条指令），我们尚未补全；本次解包的指令\
             将以 ins_N 原样显示。"
        )
    })?;

    let mut data = match file.as_str() {
        "th11.msgm" => {
            let mut d = map_file::parse_map_content("th11.msgm", TH11)?;
            merge_descriptions(&mut d, TH11_ZH);
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

    /// 核心回归：此前 `let _ = version;` 让任何版本都拿到 th17 的表。
    /// th19/th20 的 MSG 有 thmsg 独有的 th19_msg_fmts，我们没有那 16 条，
    /// 所以必须**报错而不是蒙**。
    #[test]
    fn th19_and_th20_report_missing_data_instead_of_falling_back() {
        for version in ["19", "20"] {
            let err = parse_msg_semantics(version)
                .unwrap_err();
            assert!(err.contains("th19_msg_fmts"), "要说清为什么缺: {err}");
        }
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
