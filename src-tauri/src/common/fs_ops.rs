// src-tauri/src/common/fs_ops.rs

use crate::app_state::AppState;
use std::fs;
use std::path::Path;
use tauri::State;

/// 校验目标 path 必须在 project_root 之内(canonicalize 后做前缀匹配)。
/// 防止 frontend bug / 恶意 MCP 工具调用越界文件操作。
/// 注意:path 可能不存在(create 场景),这种情况下校验**父目录**的 canonical 是否在 root 内。
fn assert_within_project(path: &str, project_root: &str) -> Result<(), String> {
    let target = Path::new(path);
    let root = Path::new(project_root);

    // root 必须能 canonicalize(项目根应该一直存在)
    let canon_root = root
        .canonicalize()
        .map_err(|e| format!("project root not accessible: {e}"))?;

    // target 可能不存在(create 场景):若不存在,校验父目录
    let canon_target = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| format!("path not accessible: {e}"))?
    } else {
        // 父目录必须存在且在 root 内
        let parent = target
            .parent()
            .ok_or_else(|| "target has no parent directory".to_string())?;
        let canon_parent = parent
            .canonicalize()
            .map_err(|e| format!("parent dir not accessible: {e}"))?;
        // 把 file_name 拼回去(canonicalize 不需要真实存在的最后一段)
        let file_name = target
            .file_name()
            .ok_or_else(|| "target has no file name".to_string())?;
        canon_parent.join(file_name)
    };

    if !canon_target.starts_with(&canon_root) {
        return Err(format!(
            "path escapes project root: {:?} not under {:?}",
            canon_target, canon_root
        ));
    }
    Ok(())
}

/// 给每个 fs 命令一个统一前缀守卫。读 AppState 的当前项目根,无根则拒绝。
pub(crate) fn guard_path(state: &AppState, path: &str) -> Result<String, String> {
    let root_guard = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let root = root_guard
        .as_deref()
        .ok_or_else(|| "no project root set, file operations refused".to_string())?;
    assert_within_project(path, root)?;
    Ok(root.to_string())
}

/// 校验**单段**文件名(不是路径!)合法性。
/// 与前端 `useFileOperations.js#validateFileName` 同义,作为后端兜底,
/// 防止前端被绕过(MCP 工具 / 第三方调用 / webview XSS)。
///
/// 拒绝:空/全空白/`.`/`..`、前后空白、路径分隔符、Windows 禁用字符
/// (`\/:*?"<>|`)、控制字符(含 NUL)、Windows 保留设备名
/// (CON/PRN/AUX/NUL/COM1-9/LPT1-9,大小写无关,带或不带后缀)。
pub(crate) fn validate_basename(name: &str) -> Result<(), String> {
    if name.trim().is_empty() || name == "." || name == ".." {
        return Err("file name cannot be empty, '.', or '..'".to_string());
    }
    if name != name.trim() {
        return Err("file name cannot have leading/trailing whitespace".to_string());
    }
    for ch in name.chars() {
        if ch.is_control() || "\\/:*?\"<>|".contains(ch) {
            return Err(format!("file name contains forbidden character: {ch:?}"));
        }
    }
    // Windows reserved names (case-insensitive, with or without extension)
    let stem = name.split('.').next().unwrap_or("").to_uppercase();
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.contains(&stem.as_str()) {
        return Err(format!("file name uses Windows reserved name: {stem}"));
    }
    Ok(())
}

/// 从完整路径取末段 basename 用于校验。
fn basename_of(path: &str) -> Result<&str, String> {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("path has no file name component: {path}"))
}

#[tauri::command]
pub fn create_directory(state: State<AppState>, path: String) -> Result<(), String> {
    guard_path(&state, &path)?;
    let name = basename_of(&path)?;
    validate_basename(name)?;
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_file(state: State<AppState>, path: String) -> Result<(), String> {
    guard_path(&state, &path)?;
    let name = basename_of(&path)?;
    validate_basename(name)?;
    // 用 O_EXCL (create_new) 原子地创建,避免 TOCTOU 双击新建覆盖。
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("Failed to create file (may already exist): {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn rename_entry(
    state: State<AppState>,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    guard_path(&state, &old_path)?;
    guard_path(&state, &new_path)?;
    let new_name = basename_of(&new_path)?;
    validate_basename(new_name)?;
    // POSIX rename 会静默覆盖目标,显式拒绝。
    if Path::new(&new_path).exists() {
        return Err(format!("Destination already exists: {new_path}"));
    }
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_entry(
    state: State<AppState>,
    source_path: String,
    destination_path: String,
) -> Result<(), String> {
    guard_path(&state, &source_path)?;
    guard_path(&state, &destination_path)?;

    let source = Path::new(&source_path);
    let destination = Path::new(&destination_path);

    if !source.exists() {
        return Err("Source path does not exist".to_string());
    }

    if destination.exists() {
        return Err("Destination already exists".to_string());
    }

    if source.is_dir() {
        copy_dir_all(source, destination).map_err(|e| e.to_string())
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(source, destination)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, path: String) -> Result<(), String> {
    guard_path(&state, &path)?;
    let path_obj = Path::new(&path);

    // 检查路径是否存在
    if !path_obj.exists() {
        return Err("Path does not exist".to_string());
    }

    // 根据是文件还是文件夹选择不同的删除策略
    if path_obj.is_dir() {
        // 递归删除文件夹
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        // 删除单个文件
        fs::remove_file(path).map_err(|e| e.to_string())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryStat {
    pub exists: bool,
    pub is_dir: bool,
    pub size: u64,
}

/// # Security note
/// 此命令**故意**不调用 guard_path,因为系统剪贴板可能复制 root 外的文件来粘贴进 root。
/// 副作用:webview XSS 可以用它探测主机任意路径的存在性 + 文件大小。仅信息泄露,
/// 不能读内容。当前威胁模型下可接受;若 webview 引入不可信内容(markdown 预览
/// 加载远程图片等)需重新评估。
#[tauri::command]
pub fn stat_entry(state: State<AppState>, path: String) -> Result<EntryStat, String> {
    // 此命令可以接受 root 外的路径(系统剪贴板可能复制 root 外的文件来粘贴进 root)
    // 所以**不**调 guard_path。但仍要求 project root 已设置以保证 frontend 是在项目语境下用它。
    let _root_guard = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let p = Path::new(&path);
    if !p.exists() {
        return Ok(EntryStat {
            exists: false,
            is_dir: false,
            size: 0,
        });
    }
    let meta = std::fs::symlink_metadata(p).map_err(|e| format!("stat failed: {e}"))?;
    Ok(EntryStat {
        exists: true,
        is_dir: meta.is_dir(),
        size: if meta.is_file() { meta.len() } else { 0 },
    })
}

/// Windows-only reparse-point 检测。`file_type().is_symlink()` 在 Windows 上
/// 只能识别"真"符号链接,**漏识 junction(`mklink /J`)和其他 reparse 类型**,
/// 攻击面是:在项目里塞一个 junction 指向 `C:\Windows`,copy 时跟过去把系统文件
/// 复制到 dst。直接读 `FILE_ATTRIBUTE_REPARSE_POINT (0x400)` 一并拦掉。
#[cfg(windows)]
fn is_reparse_point(entry: &std::fs::DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    entry
        .metadata()
        .map(|m| m.file_attributes() & 0x400 != 0)
        .unwrap_or(false)
}
#[cfg(not(windows))]
fn is_reparse_point(_entry: &std::fs::DirEntry) -> bool {
    false
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        // entry.file_type() 在 Unix 用 lstat,Windows 上同样不跟随符号链接,
        // 这样符号链接本身会被 is_symlink() 识别而不是按目标类型处理。
        let ft = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ft.is_symlink() || is_reparse_point(&entry) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("refusing to copy reparse point or symlink: {:?}", src_path),
            ));
        }
        if ft.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as stdfs;
    use tempfile::tempdir;

    #[test]
    fn assert_within_project_accepts_path_inside() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        let file = dir.path().join("a.txt");
        stdfs::write(&file, b"hi").unwrap();
        let res = assert_within_project(file.to_str().unwrap(), root);
        assert!(res.is_ok(), "expected Ok, got {res:?}");
    }

    #[test]
    fn assert_within_project_rejects_escape() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        // /etc/passwd 应当不在 tempdir 之下
        let res = assert_within_project("/etc/passwd", root);
        assert!(res.is_err(), "expected Err, got {res:?}");
        let msg = res.unwrap_err();
        assert!(
            msg.contains("escapes project root"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn assert_within_project_rejects_dotdot() {
        let dir = tempdir().unwrap();
        // 在 root 下造一个真实子目录,然后构造 ${root}/sub/../../outside
        let sub = dir.path().join("sub");
        stdfs::create_dir(&sub).unwrap();
        let root = dir.path().to_str().unwrap();
        // outside 路径:sub/../../outside.txt -> 等价 dir 的兄弟节点 outside.txt
        let escape = sub.join("..").join("..").join("outside.txt");
        let res = assert_within_project(escape.to_str().unwrap(), root);
        assert!(res.is_err(), "expected Err, got {res:?}");
        let msg = res.unwrap_err();
        assert!(
            msg.contains("escapes project root"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn assert_within_project_allows_nonexistent_target_under_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        let target = dir.path().join("newfile.txt"); // 尚未创建
        assert!(!target.exists());
        let res = assert_within_project(target.to_str().unwrap(), root);
        assert!(res.is_ok(), "expected Ok, got {res:?}");
    }

    #[test]
    fn create_file_create_new_refuses_overwrite() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("x.txt");
        stdfs::write(&target, b"existing").unwrap();
        // 直接复用底层 OpenOptions(create_file 命令本身需要 State<AppState>,这里只测原子语义)
        let res = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target);
        assert!(res.is_err(), "expected create_new to refuse existing file");
        // 原内容未被破坏
        let content = stdfs::read(&target).unwrap();
        assert_eq!(content, b"existing");
    }

    #[cfg(unix)]
    #[test]
    fn copy_dir_all_refuses_symlink() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        stdfs::create_dir(&src).unwrap();
        let data = src.join("data");
        stdfs::write(&data, b"hello").unwrap();
        let link = src.join("link");
        symlink(&data, &link).unwrap();

        let dst = dir.path().join("dst");
        let res = copy_dir_all(&src, &dst);
        assert!(res.is_err(), "expected Err, got {res:?}");
        let msg = res.unwrap_err().to_string();
        assert!(
            msg.contains("refusing to copy reparse point or symlink"),
            "unexpected error: {msg}"
        );
    }

    // ---- validate_basename tests ----

    #[test]
    fn validate_basename_rejects_separator() {
        assert!(validate_basename("foo/bar").is_err());
        assert!(validate_basename("foo\\bar").is_err());
    }

    #[test]
    fn validate_basename_rejects_colon() {
        // 拦掉 Windows ADS(`file:stream`)与盘符注入
        assert!(validate_basename("file:stream").is_err());
        assert!(validate_basename("C:foo").is_err());
    }

    #[test]
    fn validate_basename_rejects_reserved_names() {
        for n in ["CON", "con", "Con", "NUL", "NUL.txt", "com1", "LPT9", "AUX.log"] {
            let res = validate_basename(n);
            assert!(res.is_err(), "expected {n:?} rejected, got {res:?}");
        }
    }

    #[test]
    fn validate_basename_accepts_normal_names() {
        for n in [
            "foo.txt",
            "with-dash",
            "with_underscore.ext",
            "中文名.ecl",
            "th16.std",
            "a.b.c.d",
        ] {
            let res = validate_basename(n);
            assert!(res.is_ok(), "expected {n:?} accepted, got {res:?}");
        }
    }

    #[test]
    fn validate_basename_rejects_dot_and_dotdot_and_whitespace() {
        assert!(validate_basename("").is_err());
        assert!(validate_basename(".").is_err());
        assert!(validate_basename("..").is_err());
        assert!(validate_basename("  ").is_err());
        assert!(validate_basename(" leading").is_err());
        assert!(validate_basename("trailing ").is_err());
    }

    #[test]
    fn validate_basename_rejects_control_chars() {
        assert!(validate_basename("foo\0bar").is_err());
        assert!(validate_basename("foo\nbar").is_err());
        assert!(validate_basename("foo\tbar").is_err());
    }
}
