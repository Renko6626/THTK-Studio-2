use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

// Windows 专用的常量，用于隐藏控制台窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 定义命令执行的结果结构体
#[derive(Debug, Serialize, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// 核心函数：运行外部工具
///
/// - `exe_path`: 可执行文件的绝对路径
/// - `args`: 参数列表
/// - `working_dir`: 可选的工作目录。如果不填，默认继承当前进程或由系统决定（通常建议传入 thtk 所在的目录或项目目录）
pub fn run_tool(
    exe_path: &str,
    args: &[&str],
    working_dir: Option<&Path>,
) -> Result<CommandResult, String> {
    // 1. 构建命令
    let mut cmd = Command::new(exe_path);
    cmd.args(args);

    // 2. 设置工作目录 (Context)
    // 这对于 thtk 很重要，因为 thtk 经常需要读取同目录下的 .eclmap 或配置
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // 3. 隐藏 Windows 下的 CMD 弹窗
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // 4. 执行命令
    // map_err 处理的是 "找不到 exe" 这种系统级错误
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to spawn process '{}': {}", exe_path, e))?;

    // 5. 解码输出 (Shift-JIS -> UTF-8)
    // 无论 thtk 输出什么，我们都尝试用 Shift-JIS 解码，这样能保证日文报错正常显示
    let (stdout_cow, _, _) = SHIFT_JIS.decode(&output.stdout);
    let (stderr_cow, _, _) = SHIFT_JIS.decode(&output.stderr);

    Ok(CommandResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout: stdout_cow.into_owned(),
        stderr: stderr_cow.into_owned(),
    })
}

/// 辅助函数：专门用于检查 thtk 是否可用
/// 可以用来在设置界面验证用户输入的路径是否正确
pub fn check_tool_version(exe_path: &str) -> bool {
    // 尝试运行 "tool.exe --help" 或类似命令
    // thtk 的工具通常没有统一的版本参数，但一般不带参数运行会输出 help 且 exit code 为 1 或 0
    // 这里我们简单尝试调用一下，看是否能启动
    match run_tool(exe_path, &[], None) {
        Ok(_) => true, // 只要能跑起来，不管返回啥，都说明路径是对的
        Err(_) => false,
    }
}
