use crate::common::toolchain;
use crate::config::AppConfig;
use crate::modules::ecl::error_parser::Diagnostic;
use crate::utils;
use crate::common::text_encoding;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

// Windows 专用的常量，用于隐藏控制台窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MsgMode {
    Decompile,
    Compile,
}

impl MsgMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MsgMode::Decompile => "decompile",
            MsgMode::Compile => "compile",
        }
    }
}

fn default_with_comments() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgRequest {
    pub mode: MsgMode,
    pub version: String,
    pub input_path: String,
    pub output_path: Option<String>,
    /// decompile 时是否在每行末尾追加 ` // <description>`;compile 路径忽略此字段
    #[serde(default = "default_with_comments")]
    pub with_comments: bool,
    /// 游戏文本的编码。解包时用它解 thmsg 吐出的字节，打包时用它编码写回。
    ///
    /// 两个方向**各自独立**：常见工作流是按 shift-jis 解开原版日文，翻译后按
    /// gbk 打包成汉化版。留空则回退项目配置、再回退 shift-jis。
    #[serde(default)]
    pub encoding: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgResult {
    pub success: bool,
    pub tool: String,        // "thmsg"
    pub mode: String,        // "decompile" | "compile"
    pub script_kind: String, // "msg"
    pub input_path: String,
    pub output_path: Option<String>,
    pub message: String,
    /// 始终为空;复用 ECL 的 Diagnostic 类型以便前端面板复用。
    pub diagnostics: Vec<Diagnostic>,
}

/// thmsg 与 thecl 版本归一化一致:`th17` → `17`,`17` → `17`。
pub fn normalize_thmsg_version(version: &str) -> String {
    let trimmed = version.trim().to_lowercase();
    if let Some(stripped) = trimmed.strip_prefix("th") {
        return stripped.to_string();
    }
    trimmed
}

/// 推断输出路径:
///   - decompile: `.msg` → `.dmsg`(同目录同 stem)
///   - compile:   `.dmsg` → `.msg`
///   - 找不到对应扩展则直接追加。
pub fn infer_output_path(request: &MsgRequest) -> String {
    if let Some(output_path) = &request.output_path {
        return output_path.clone();
    }
    match request.mode {
        MsgMode::Decompile => swap_or_append_extension(&request.input_path, ".msg", ".dmsg"),
        MsgMode::Compile => swap_or_append_extension(&request.input_path, ".dmsg", ".msg"),
    }
}

fn swap_or_append_extension(path: &str, from: &str, to: &str) -> String {
    let replaced = path.replace(from, to);
    if replaced == path {
        format!("{}{}", path, to)
    } else {
        replaced
    }
}

fn fail(request: &MsgRequest, msg: impl Into<String>) -> MsgResult {
    MsgResult {
        success: false,
        tool: "thmsg".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "msg".to_string(),
        input_path: request.input_path.clone(),
        output_path: None,
        message: msg.into(),
        diagnostics: Vec::new(),
    }
}

/// 生成跨调用唯一的临时文件路径(进程 ID + 调用级计数器),
/// 模式与 modules/mcp/tools.rs::check_ecl 一致。
fn unique_temp_path(stem: &str, ext: &str) -> std::path::PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "thtk-msg-{}-{seq}-{stem}.{ext}",
        std::process::id()
    ))
}

/// 构造 std::process::Command 并应用 Windows 下的 CREATE_NO_WINDOW
/// 与 cmd_runner 行为一致(避免 GUI 进程下闪 cmd 窗口)。
fn build_command(exe_path: &str, args: &[&str], work_dir: Option<&Path>) -> Command {
    let mut cmd = Command::new(exe_path);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn run(config: &AppConfig, request: &MsgRequest) -> MsgResult {
    let tool_path = toolchain::resolve_tool_path(config, "thmsg", "thmsg.exe");
    if tool_path.trim().is_empty() {
        return fail(request, &crate::common::toolchain::not_configured_message("thmsg"));
    }

    match request.mode {
        MsgMode::Decompile => run_decompile(&tool_path, request),
        MsgMode::Compile => run_compile(&tool_path, request),
    }
}

/// 本次调用生效的编码：请求里指定的优先，留空回退 shift-jis。
///
/// 项目级默认值在 `commands.rs` 里填进 request，这里只做最后兜底——
/// compiler 是纯函数层，不该自己去读项目配置。
fn effective_encoding(request: &MsgRequest) -> String {
    let raw = request.encoding.trim();
    if raw.is_empty() {
        "shift-jis".to_string()
    } else {
        raw.to_lowercase()
    }
}

fn run_decompile(tool_path: &str, request: &MsgRequest) -> MsgResult {
    let version = normalize_thmsg_version(&request.version);
    let output_path = infer_output_path(request);
    let input_parent = Path::new(&request.input_path).parent();

    // 1. 跑 `thmsg -d <ver> <input>`,抓 raw stdout bytes(SJIS dmsg 文本)+ stderr string。
    let args = ["-d", version.as_str(), request.input_path.as_str()];
    let output = match build_command(tool_path, &args, input_parent).output() {
        Ok(o) => o,
        Err(e) => return fail(request, format!("Failed to launch thmsg: {e}")),
    };
    let stderr_str = SHIFT_JIS.decode(&output.stderr).0.into_owned();

    if !output.status.success() {
        let mut msg = String::new();
        if let Some(code) = output.status.code() {
            msg.push_str(&format!("thmsg exited with code {code}\n"));
        }
        msg.push_str(&stderr_str);
        return fail(request, msg);
    }
    if output.stdout.is_empty() {
        let mut msg = String::from("thmsg returned empty stdout\n");
        msg.push_str(&stderr_str);
        return fail(request, msg);
    }

    // 2. 按指定编码解成 UTF-8。不能无脑用 SJIS——汉化版的 .msg 常是 GBK，
    //    而 SJIS 解 GBK 字节不会报任何错，只会安静地给出满屏乱码。
    let encoding = effective_encoding(request);
    let (raw_dmsg, mojibake_warning) =
        text_encoding::decode_with_warning(&output.stdout, &encoding);

    // 3. 语义加载 + 翻译
    // 拿不到语义不该让整个解包失败——用户仍然需要看到 ins_N 形式的内容。
    // 但必须说清楚为什么没有名字（th19/th20 的 MSG 正处于这个状态）。
    let (semantics, semantics_note) = match super::map_parser::parse_msg_semantics(&version) {
        Ok(s) => (Some(s), None),
        Err(e) => (None, Some(e)),
    };
    let readable = match &semantics {
        Some(s) => super::translator::dmsg_to_readable(&raw_dmsg, s, request.with_comments),
        None => raw_dmsg.clone(),
    };

    // 4. 写 UTF-8 到目标 .dmsg
    if let Err(e) = utils::write_file_utf8(&output_path, &readable) {
        return fail(request, format!("Failed to write {output_path}: {e}"));
    }

    let mut message = if stderr_str.trim().is_empty() {
        format!("Decompiled {} → {}（编码 {encoding}）", request.input_path, output_path)
    } else {
        stderr_str
    };
    // 解包本身几乎不会失败，失败的是"解得对不对"。可疑就说出来，让用户换编码重解。
    if let Some(warning) = mojibake_warning {
        message.push_str("\n\n⚠ ");
        message.push_str(&warning);
    }
    // 语义表缺失是"能用但没名字"，属于提示不是错误——必须让用户看见，
    // 否则他会以为这些 ins_N 就是本来的样子。
    if let Some(note) = semantics_note {
        message.push_str("\n\nℹ ");
        message.push_str(&note);
    }

    MsgResult {
        success: true,
        tool: "thmsg".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "msg".to_string(),
        input_path: request.input_path.clone(),
        output_path: Some(output_path),
        message,
        diagnostics: Vec::new(),
    }
}

fn run_compile(tool_path: &str, request: &MsgRequest) -> MsgResult {
    let version = normalize_thmsg_version(&request.version);
    let output_path = infer_output_path(request);

    // 1. 读源文件。正常是我们自己写的 UTF-8；用户手搓的 .dmsg 可能直接是
    //    游戏编码，故以本次请求的编码兜底。
    let content = match utils::read_text_file(&request.input_path, &effective_encoding(request)) {
        Ok(c) => c,
        Err(e) => {
            return fail(
                request,
                format!("Failed to read {}: {e}", request.input_path),
            )
        }
    };

    // 2. 翻译回 raw dmsg
    let semantics = match super::map_parser::parse_msg_semantics(&version) {
        Ok(s) => s,
        Err(e) => return fail(request, format!("Failed to load msg semantics: {e}")),
    };
    let raw_dmsg = super::translator::readable_to_dmsg(&content, &semantics);

    // 3. UTF-8 → 目标编码。装不下**必须失败**：encoding_rs 的 encode 对无法映射的
    //    字符不报错，而是写入 HTML 数字实体（"你" → "&#20320;"），打包会"成功"
    //    而游戏里显示的是这串实体本身。更隐蔽的是简体汉字大部分能编进 Shift-JIS
    //    （"好世界再朋友"日文汉字里都有），只有少数字变实体，症状是"大部分正常、
    //    个别地方冒怪东西"。
    let encoding = effective_encoding(request);
    let sjis_bytes = match text_encoding::encode_strict(&raw_dmsg, &encoding) {
        Ok(bytes) => bytes,
        Err(offenders) => {
            return fail(
                request,
                text_encoding::describe_unmappable(&offenders, &encoding),
            )
        }
    };
    let stem = Path::new(&request.input_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());
    let temp_path = unique_temp_path(&stem, "dmsg.raw");
    if let Err(e) = std::fs::write(&temp_path, &sjis_bytes) {
        return fail(
            request,
            format!("Failed to write temp file {}: {e}", temp_path.display()),
        );
    }

    // 4. 跑 `thmsg -c <ver> <temp> <out>`
    let temp_str = temp_path.to_string_lossy().to_string();
    let args = ["-c", version.as_str(), temp_str.as_str(), output_path.as_str()];
    let work_dir = Path::new(&output_path).parent();
    let output = match build_command(tool_path, &args, work_dir).output() {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return fail(request, format!("Failed to launch thmsg: {e}"));
        }
    };
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup

    let stdout_str = SHIFT_JIS.decode(&output.stdout).0.into_owned();
    let stderr_str = SHIFT_JIS.decode(&output.stderr).0.into_owned();
    let combined = format!("{stdout_str}\n{stderr_str}").trim().to_string();

    if !output.status.success() {
        let mut msg = String::new();
        if let Some(code) = output.status.code() {
            msg.push_str(&format!("thmsg exited with code {code}\n"));
        }
        msg.push_str(&combined);
        return fail(request, msg);
    }

    let message = if combined.is_empty() {
        format!("Compiled {} → {}", request.input_path, output_path)
    } else {
        combined
    };

    MsgResult {
        success: true,
        tool: "thmsg".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "msg".to_string(),
        input_path: request.input_path.clone(),
        output_path: Some(output_path),
        message,
        diagnostics: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(mode: MsgMode, input: &str, output: Option<&str>) -> MsgRequest {
        MsgRequest {
            mode,
            version: "17".to_string(),
            input_path: input.to_string(),
            output_path: output.map(|s| s.to_string()),
            with_comments: true,
            encoding: String::new(),
        }
    }

    #[test]
    fn infer_output_path_decompile_swaps_msg_to_dmsg() {
        let r = req(MsgMode::Decompile, "/proj/st01.msg", None);
        assert_eq!(infer_output_path(&r), "/proj/st01.dmsg");
    }

    #[test]
    fn infer_output_path_compile_swaps_dmsg_to_msg() {
        let r = req(MsgMode::Compile, "/proj/st01.dmsg", None);
        assert_eq!(infer_output_path(&r), "/proj/st01.msg");
    }

    #[test]
    fn infer_output_path_decompile_without_msg_extension_appends() {
        // 没有 .msg 也不该原样吞掉:应追加 .dmsg。
        let r = req(MsgMode::Decompile, "/proj/strange_file", None);
        assert_eq!(infer_output_path(&r), "/proj/strange_file.dmsg");
    }

    #[test]
    fn infer_output_path_respects_explicit_override() {
        let r = req(MsgMode::Decompile, "/proj/st01.msg", Some("/out/here.txt"));
        assert_eq!(infer_output_path(&r), "/out/here.txt");
    }

    #[test]
    fn normalize_thmsg_version_strips_th_prefix() {
        assert_eq!(normalize_thmsg_version("th17"), "17");
        assert_eq!(normalize_thmsg_version("17"), "17");
        assert_eq!(normalize_thmsg_version("  Th18 "), "18");
    }

    #[test]
    fn msg_request_roundtrip_serde_defaults_with_comments_true() {
        // 缺省 withComments 字段时,反序列化应填 true。
        let json = r#"{"mode":"decompile","version":"17","inputPath":"/x.msg"}"#;
        let parsed: MsgRequest = serde_json::from_str(json).expect("parse");
        assert!(matches!(parsed.mode, MsgMode::Decompile));
        assert_eq!(parsed.version, "17");
        assert_eq!(parsed.input_path, "/x.msg");
        assert!(parsed.output_path.is_none());
        assert!(parsed.with_comments, "default should be true");

        // 反向再序列化 + 再反序列化应保持语义。
        let s = serde_json::to_string(&parsed).expect("ser");
        let parsed2: MsgRequest = serde_json::from_str(&s).expect("reparse");
        assert!(parsed2.with_comments);
        assert!(matches!(parsed2.mode, MsgMode::Decompile));
    }

    #[test]
    fn run_returns_failure_when_thmsg_path_not_configured() {
        // 用默认 AppConfig(thtk_dir 空、tool_overrides 空)走 run(),应直接拿到 success:false。
        let cfg = AppConfig::default();
        let r = req(MsgMode::Decompile, "/proj/st01.msg", None);
        let result = run(&cfg, &r);
        assert!(!result.success);
        assert!(
            result.message.contains("thmsg") && result.message.contains("工具链设置"),
            "got message: {}",
            result.message
        );
        assert!(result.output_path.is_none());
        assert_eq!(result.tool, "thmsg");
        assert_eq!(result.mode, "decompile");
        assert_eq!(result.script_kind, "msg");
    }

    // ---- 编码：本轮修复的核心行为 ----

    #[test]
    fn effective_encoding_defaults_to_shift_jis() {
        let mut r = req(MsgMode::Compile, "a.dmsg", None);
        r.encoding = String::new();
        assert_eq!(effective_encoding(&r), "shift-jis");
        r.encoding = "   ".to_string();
        assert_eq!(effective_encoding(&r), "shift-jis");
    }

    #[test]
    fn effective_encoding_is_case_insensitive() {
        let mut r = req(MsgMode::Compile, "a.dmsg", None);
        r.encoding = "  GBK ".to_string();
        assert_eq!(effective_encoding(&r), "gbk");
    }

    /// 回归：原实现 `SHIFT_JIS.encode()` 丢掉第三个返回值，简体汉字被静默写成
    /// `&#20320;` 这样的 HTML 实体，打包"成功"而游戏里显示的就是这串实体。
    /// 现在必须失败，且错误里指出是哪些字、在第几行第几列。
    #[test]
    fn packing_chinese_as_shift_jis_fails_with_located_characters() {
        let offenders =
            crate::common::text_encoding::encode_strict("博丽灵梦", "shift-jis").unwrap_err();
        let message = crate::common::text_encoding::describe_unmappable(&offenders, "shift-jis");
        assert!(message.contains("丽") || message.contains("灵"));
        assert!(message.contains("行"), "要能定位到行: {message}");
        assert!(message.contains("GBK"), "要给出下一步: {message}");
    }

    /// 同一段中文换成 GBK 就能打包——这正是汉化版该走的路。
    #[test]
    fn packing_chinese_as_gbk_succeeds() {
        assert!(crate::common::text_encoding::encode_strict("博丽灵梦", "gbk").is_ok());
    }
}
