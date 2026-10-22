//! MSG 的逐版本签名与名字档案。
//!
//! mapfile 是**名字**的数据契约，但一张表无法表达 thmsg 的版本差异：
//! `thmsg06.c` 的 `th06_find_format()` 是按版本 fallthrough 的，同一个 opcode
//! 在 th14 / th16 / th18 上的参数形状并不相同。这里把那张分支表原样编码成
//! 增量表，加载 mapfile 后再套到当前版本上。
//!
//! 纪律与 mapfile 一致：**签名以 thtk 源码为准**。少数 thtk 自身的历史遗留
//! （例如 op15/16 在 th10+ 从未使用，却仍会从 th09/th08 的旧表命中）也照抄，
//! 因为 `thmsg` 就是这么解的；预检查若自行改写，会与实际后端行为脱节。

use crate::common::map_file::{MapFileData, MapInstruction, MapParameter};
use std::collections::BTreeMap;

type SigTable = &'static [(u32, &'static str)];

/// th06 基础表。th07–th20 都从它继承。
const TH06: SigTable = &[
    (0, ""),
    (1, "ss"),
    (2, "ss"),
    (3, "ssz"),
    (4, "S"),
    (5, "ss"),
    (6, ""),
    (7, "S"),
    (8, "ssz"),
    (9, "S"),
    (10, ""),
    (11, ""),
    (12, ""),
    (13, "S"),
    (14, ""),
];

const TH08: SigTable = &[
    (3, "ssm"),
    (8, "ssm"),
    (15, "SSSSS"),
    (16, "m"),
    (17, "SS"),
    (18, "S"),
    (19, "m"),
    (20, "m"),
    (21, "S"),
    (22, ""),
];

const TH09: SigTable = &[
    (8, ""),
    (15, "SSS"),
    (23, "S"),
    (24, ""),
    (25, ""),
    (28, "S"),
];

const TH10: SigTable = &[
    (1, "S"),
    (2, "S"),
    (3, ""),
    (4, ""),
    (5, ""),
    (7, ""),
    (10, "S"),
    (12, "S"),
    (14, "S"),
    (17, "m"),
    (18, ""),
    (19, ""),
    (20, ""),
    (21, ""),
    (22, ""),
    (23, ""),
    (25, "S"),
];

const TH11: SigTable = &[(9, ""), (10, "S"), (11, "S"), (12, "")];
const TH12: SigTable = &[(27, "f")];
const TH128: SigTable = &[(28, "ff"), (29, "S"), (30, "")];
const TH13: SigTable = &[(31, "S")];
const TH14: SigTable = &[(5, "S"), (8, "S"), (14, "SS"), (20, "S"), (32, "S")];
const TH143: SigTable = &[(33, "S")];
const TH16: SigTable = &[(19, "S"), (34, "SS"), (35, "")];
const TH18: SigTable = &[(4, "S"), (7, "S"), (13, "SS"), (36, "")];
const TH185: SigTable = &[(37, ""), (38, ""), (39, "")];
const TH19: SigTable = &[
    (42, "S"),
    (43, "S"),
    (44, "ff"),
    (45, "ff"),
    (46, "SS"),
    (47, "SS"),
    (50, "S"),
    (51, "S"),
    (52, ""),
    (53, ""),
    (54, ""),
    (55, ""),
    (56, ""),
];

// 下面每个数组都是一条 fallthrough 链，顺序与 thmsg06.c 的 switch 完全一致：
// 先命中的表赢。
const TABLES_19: &[SigTable] = &[
    TH19, TH185, TH18, TH16, TH143, TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_185: &[SigTable] = &[
    TH185, TH18, TH16, TH143, TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_18: &[SigTable] = &[
    TH18, TH16, TH143, TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_16: &[SigTable] = &[
    TH16, TH143, TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_143: &[SigTable] = &[
    TH143, TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_14: &[SigTable] = &[
    TH14, TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06,
];
const TABLES_13: &[SigTable] = &[TH13, TH128, TH12, TH11, TH10, TH09, TH08, TH06];
const TABLES_128: &[SigTable] = &[TH128, TH12, TH11, TH10, TH09, TH08, TH06];
const TABLES_12: &[SigTable] = &[TH12, TH11, TH10, TH09, TH08, TH06];
const TABLES_11: &[SigTable] = &[TH11, TH10, TH09, TH08, TH06];
const TABLES_10: &[SigTable] = &[TH10, TH09, TH08, TH06];
const TABLES_09: &[SigTable] = &[TH09, TH08, TH06];
const TABLES_08: &[SigTable] = &[TH08, TH06];
const TABLES_06: &[SigTable] = &[TH06];

const ED10: SigTable = &[
    (0, ""),
    (3, "m"),
    // thtk 的 th10_msg_ed_fmts 漏了 4/13（引擎里确实有分支，见 TH16/TH18 一手
    // 反编译），但 thmsg -c 会直接拒绝这两个 id。这里不收录，保持
    // 「表里的 opcode = thmsg 能编译的 opcode」这条不变量。
    (5, "S"),
    (6, "S"),
    (7, "Sz"),
    (8, "SSS"),
    (9, "S"),
    (10, "z"),
    (11, ""),
    (12, "z"),
    (14, "S"),
    (15, "SSS"),
    (16, "SSS"),
    (17, "SSS"),
];
const ED20: SigTable = &[(9, "SS")];

const ED_TABLES_20: &[SigTable] = &[ED20, ED10];
const ED_TABLES_10: &[SigTable] = &[ED10];

fn find(tables: &[SigTable], opcode: u32) -> Option<&'static str> {
    for table in tables {
        if let Some((_, signature)) = table.iter().find(|(op, _)| *op == opcode) {
            return Some(*signature);
        }
    }
    None
}

fn tables_for_stage(version: u32) -> &'static [SigTable] {
    match version {
        20 | 19 => TABLES_19,
        185 => TABLES_185,
        18 => TABLES_18,
        17 | 165 | 16 => TABLES_16,
        15 | 143 => TABLES_143,
        14 => TABLES_14,
        13 => TABLES_13,
        128 | 125 => TABLES_128,
        12 => TABLES_12,
        11 => TABLES_11,
        10 => TABLES_10,
        9 => TABLES_09,
        8 => TABLES_08,
        7 | 6 => TABLES_06,
        _ => &[],
    }
}

pub(crate) fn stage_signature(version: u32, opcode: u32) -> Option<&'static str> {
    find(tables_for_stage(version), opcode)
}

pub(crate) fn ending_signature(version: u32, opcode: u32) -> Option<&'static str> {
    if version >= 20 {
        find(ED_TABLES_20, opcode)
    } else {
        find(ED_TABLES_10, opcode)
    }
}

/// MSG 的参数名比通用 mapfile 更具体：`m` 是掩码文本、`z` 是明文串。
/// 通用解析器把 `m` 标成 subroutine（ECL 语义），这里覆盖掉。
fn build_msg_params(signature: &str) -> Vec<MapParameter> {
    let mut counters: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut params = Vec::new();

    for ch in signature.chars() {
        if ch.is_whitespace() {
            continue;
        }
        let type_name = match ch {
            'S' => "int",
            's' => "int16",
            'f' => "float",
            'm' => "text",
            'z' => "string",
            'C' => "color",
            'b' => "byte",
            'D' => "label",
            'o' => "offset",
            't' => "time",
            'x' => "difficulty",
            _ => continue,
        };
        let counter = counters.entry(type_name).or_insert(0);
        *counter += 1;
        params.push(MapParameter {
            name: format!("{type_name}{counter}"),
            type_name: type_name.to_string(),
        });
    }

    params
}

fn apply_signature(instruction: &mut MapInstruction, signature: Option<&'static str>) {
    match signature {
        Some(sig) if !sig.is_empty() => {
            instruction.signature = Some(sig.to_string());
            instruction.params = build_msg_params(sig);
        }
        _ => {
            instruction.signature = None;
            instruction.params.clear();
        }
    }
}

fn stage_max_opcode(version: u32) -> u32 {
    match version {
        10 => 25,
        11 => 26,
        12 | 125 => 27,
        128 => 30,
        13 => 31,
        14 | 15 => 32,
        143 | 165 => 33,
        16 | 17 => 35,
        18 => 36,
        185 => 41,
        // th20 的 37–56 没有名字，但 thtk 仍能解码；保留未命名条目以便
        // 显示 ins_N 并允许编译。
        20 => 56,
        19 => 56,
        _ => 0,
    }
}

/// 套用版本档案：先删掉该版本不存在的 opcode，再改名字特例，最后写签名。
pub fn apply_stage_profile(data: &mut MapFileData, version: u32) {
    let max = stage_max_opcode(version);
    data.instructions.retain(|instruction| instruction.opcode <= max);

    // TH143/TH165 的 33 是 tutorial，与 TH16 的 darkenPortrait 同号不同义。
    if matches!(version, 143 | 165) {
        if let Some(instruction) = data
            .instructions
            .iter_mut()
            .find(|instruction| instruction.opcode == 33)
        {
            instruction.name = "tutorial".to_string();
            instruction.description = Some("在场景类作品中启动教程。".to_string());
        }
    }

    // TH185 新增 37–39；40/41 在 thpages 有名字，但 thtk 没有任何格式表项，
    // 写进 .dmsg 会让 `thmsg -c` 直接失败，因此故意不命名（显示为 ins_N）。
    if version == 185 {
        for (opcode, name, description) in [
            (37, "bmInfoOpen", "画面变暗，表示说明文字。"),
            (38, "bmFocusInfo", "标记「教学 / 系统」在说话（取代教程图片）。"),
            (39, "bmInfoClose", "取消 bmInfoOpen 的变暗。"),
        ] {
            if !data
                .instructions
                .iter()
                .any(|instruction| instruction.opcode == opcode)
            {
                data.instructions.push(MapInstruction {
                    opcode,
                    name: name.to_string(),
                    section: None,
                    signature: None,
                    params: Vec::new(),
                    description: Some(description.to_string()),
                });
            }
        }
        data.instructions.sort_by_key(|instruction| instruction.opcode);
    }

    for instruction in &mut data.instructions {
        apply_signature(
            instruction,
            stage_signature(version, instruction.opcode),
        );
    }

    // thtk 的 th16_msg_fmts 把 musicBoss 记成带一个 S，但 TH16–TH18 的真实文件
    // 里这条是 **size 0**（曲目索引是 TH185 才加的）。thmsg -d 因此输出裸 `19`，
    // 而 thmsg -c 又按表要求一个参数——上游 dump/compile 自相矛盾。
    //
    // IDE 以真实文件为准：这里的签名给 0 个参数，编译时再由
    // `translator::apply_thtk_quirks` 补一个被引擎忽略的 1。
    if matches!(version, 16 | 17 | 165 | 18) {
        if let Some(instruction) = data
            .instructions
            .iter_mut()
            .find(|instruction| instruction.opcode == 19)
        {
            apply_signature(instruction, Some(""));
        }
    }
}

pub fn apply_ending_profile(data: &mut MapFileData, version: u32) {
    for instruction in &mut data.instructions {
        apply_signature(
            instruction,
            ending_signature(version, instruction.opcode),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_signatures_follow_thmsg_fallthrough() {
        // TH13 的 31 是新指令，32 还不存在。
        assert_eq!(stage_signature(13, 31), Some("S"));
        assert_eq!(stage_signature(13, 32), None);

        // TH14 把 5/8/14/20 换成带 who 的版本，并新增 32。
        assert_eq!(stage_signature(14, 5), Some("S"));
        assert_eq!(stage_signature(14, 14), Some("SS"));
        assert_eq!(stage_signature(14, 32), Some("S"));

        // TH16 新增 34/35；4/7/13 仍沿用旧表（无参 / 单参）。
        assert_eq!(stage_signature(16, 34), Some("SS"));
        assert_eq!(stage_signature(16, 4), Some(""));
        assert_eq!(stage_signature(16, 13), Some("S"));

        // TH18 把 4/7/13 改成带 who 的版本。
        assert_eq!(stage_signature(18, 4), Some("S"));
        assert_eq!(stage_signature(18, 7), Some("S"));
        assert_eq!(stage_signature(18, 13), Some("SS"));

        // TH185 新增 37–39，并把 19 变成带曲目索引。
        assert_eq!(stage_signature(185, 19), Some("S"));
        // TH16–18 的真实文件里 musicBoss 是零参数（见 apply_stage_profile）。
        assert_eq!(stage_signature(18, 19), Some("S"));
        assert_eq!(stage_signature(185, 37), Some(""));

        // TH19 的对战作专用指令。
        assert_eq!(stage_signature(19, 44), Some("ff"));
        assert_eq!(stage_signature(19, 46), Some("SS"));

        // thtk 的 fallthrough 在未使用的 15/16 上会命中 th09/th08 的旧表。
        // 照抄，不自行改写——预检查必须与 thmsg 实际行为一致。
        assert_eq!(stage_signature(17, 15), Some("SSS"));
        assert_eq!(stage_signature(17, 16), Some("m"));
    }

    #[test]
    fn ending_signatures_are_th10_shared_with_th20_override() {
        assert_eq!(ending_signature(18, 0), Some(""));
        assert_eq!(ending_signature(18, 7), Some("Sz"));
        assert_eq!(ending_signature(18, 9), Some("S"));
        // thtk 的结局表没有 4/13；不把它们当成可编译指令。
        assert_eq!(ending_signature(18, 4), None);
        assert_eq!(ending_signature(18, 13), None);
        assert_eq!(ending_signature(20, 9), Some("SS"));
        // TH20 没有覆盖的条目继续走 th10 表。
        assert_eq!(ending_signature(20, 7), Some("Sz"));
    }

    #[test]
    fn msg_params_use_domain_names() {
        let params = build_msg_params("Sz");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].type_name, "int");
        assert_eq!(params[1].type_name, "string");

        let text = build_msg_params("m");
        assert_eq!(text[0].type_name, "text");
    }
}
