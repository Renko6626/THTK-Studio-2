//! STD 语义数据的加载。
//!
//! 数据是外挂的 `.stdm`（eclmap 家族格式，与 ExpHP/truth 的 `map/` 互换），
//! 版本→表的映射由 `any.stdm` 这个 gamemap 决定。
//!
//! **与 MSG 的关键差异**：`thstd.c` 把版本映射为三档 `formats_v0/v1/v2` 且
//! **无 per-version 分支**，其中 `formats_v2` 覆盖 th14–th20。所以这一档共用
//! 一份表是有源码依据的——th19/th20 可以放心列进 gamemap，不像 MSG 那样存在
//! 版本独有的签名表。

use crate::common::{game_map, map_file};
use std::collections::HashMap;

const GAMEMAP: &str = include_str!("../../../assets/maps/any.stdm");
const TH14: &str = include_str!("../../../assets/maps/th14.stdm");
const TH14_ZH: &str = include_str!("../../../assets/maps/th14.stdm.zh.json");

/// 把旁挂的中文说明合并进指令表。mapfile 本体保持与生态一致，中文是我们的增量。
fn merge_descriptions(data: &mut map_file::MapFileData, zh_json: &str) {
    let map: HashMap<String, String> = match serde_json::from_str(zh_json) {
        Ok(m) => m,
        Err(_) => return,
    };
    for ins in &mut data.instructions {
        if let Some(text) = map.get(&ins.opcode.to_string()) {
            ins.description = Some(text.clone());
        }
    }
}

/// 按版本取 STD 语义数据。**查不到就报错**，不回退到任意一份表。
pub fn parse_std_semantics(version: &str) -> Result<map_file::MapFileData, String> {
    let id: u32 = version
        .trim()
        .parse()
        .map_err(|_| format!("版本号非法: {version:?}"))?;

    let file = game_map::resolve_map_file(GAMEMAP, id).ok_or_else(|| {
        format!(
            "尚无 th{id} 的 STD 指令表（目前覆盖 th14–th20，即 thstd 的 formats_v2 一档）；\
             本次解包的指令将以 ins_N 原样显示。"
        )
    })?;

    let mut data = match file.as_str() {
        "th14.stdm" => {
            let mut d = map_file::parse_map_content("th14.stdm", TH14)?;
            merge_descriptions(&mut d, TH14_ZH);
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
        let data = parse_std_semantics("17").expect("th17 应有数据");
        let jmp = data
            .instructions
            .iter()
            .find(|i| i.name == "jmp")
            .expect("jmp 必须在表里");
        assert_eq!(jmp.opcode, 1);
    }

    /// 与 MSG 相反：thstd 的 formats_v2 覆盖 th14–th20 且无 per-version 分支，
    /// 所以 th19/th20 **应该**能拿到数据。这条钉住两者的差异。
    #[test]
    fn th19_and_th20_are_covered_unlike_msg() {
        for version in ["19", "20"] {
            assert!(
                parse_std_semantics(version).is_ok(),
                "thstd 的 formats_v2 覆盖 th14–th20，th{version} 应有数据"
            );
        }
    }

    #[test]
    fn pre_th14_reports_missing_instead_of_falling_back() {
        // th13 属于 formats_v1，指令表不同；我们还没迁移那一档
        let err = parse_std_semantics("13").unwrap_err();
        assert!(err.contains("th14"), "要说清覆盖范围: {err}");
    }

    #[test]
    fn descriptions_are_merged_from_the_sidecar() {
        let data = parse_std_semantics("17").unwrap();
        let jmp = data.instructions.iter().find(|i| i.opcode == 1).unwrap();
        assert!(jmp.description.is_some(), "中文说明应合并进来");
    }

    #[test]
    fn garbage_version_is_rejected() {
        assert!(parse_std_semantics("abc").is_err());
    }
}
