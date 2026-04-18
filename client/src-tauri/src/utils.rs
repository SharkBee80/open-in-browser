//! 实用工具函数模块
//!
//! 提供浏览器启动等通用功能。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::client::bad_request;
use std::process::Command;

// 命令黑名单 - 任何危害操作系统的可执行文件或参数
const COMMAND_BLACKLIST: &[&str] = &[
    // 系统命令解释器
    "cmd.exe",
    "cmd",
    "powershell.exe",
    "powershell",
    "pwsh",
    "bash.exe",
    "bash",
    "sh.exe",
    "sh",
    "zsh",
    "command.com",
    // 脚本执行器
    "wscript.exe",
    "wscript",
    "cscript.exe",
    "cscript",
    "mshta.exe",
    "mshta",
    "certutil.exe",
    "certutil",
    // 系统管理工具
    "reg.exe",
    "reg",
    "regedit.exe",
    "regedit",
    "schtasks.exe",
    "schtasks",
    "net.exe",
    "net",
    "net1.exe",
    "sc.exe",
    "sc",
    "wmic.exe",
    "wmic",
    "taskkill.exe",
    "taskkill",
    "tasklist.exe",
    "tasklist",
    "shutdown.exe",
    "shutdown",
    "format.com",
    "format",
    "diskpart.exe",
    "diskpart",
    // 远程执行工具
    "psexec.exe",
    "psexec",
    "winrm.exe",
    "winrm",
    // 危险参数模式
    "--exec",
    "--execute",
    "--eval",
    "/c",
    "/k",
    "/r", // cmd.exe 参数
    "-Command",
    "-EncodedCommand", // PowerShell 参数
];

// 危险参数前缀（用于匹配参数中的危险模式）
const DANGEROUS_ARG_PATTERNS: &[&str] = &[
    "cmd.exe",
    "powershell",
    "bash",
    "sh.exe",
    "wscript",
    "cscript",
    "mshta",
    "regedit",
    "schtasks",
    "taskkill",
    "shutdown",
    "format",
    "diskpart",
];

/// 解析并验证命令列表（包括语法解析和黑名单检查）
///
/// # 参数
/// * `cmd_strings` - 原始命令字符串列表
///
/// # 返回
/// 成功时返回解析后的命令参数列表 (`Vec<Vec<String>>`)，  
/// 失败时返回对应的 HTTP 错误响应 (`Response`)
pub fn check_command(cmd_strings: &[String]) -> Result<Vec<Vec<String>>, Response> {
    // 先解析并检查所有命令的黑名单
    let mut parsed_commands = Vec::new();

    for cmd_str in cmd_strings {
        // 使用 shell_words 切割命令（支持引号和转义）
        let parts =
            match shell_words::split(cmd_str.replace("\\\\", "/").replace("\\", "/").as_str()) {
                Ok(p) => p,
                Err(e) => {
                    let msg = format!("Failed to parse command '{}': {}", cmd_str, e);
                    return Err(bad_request(msg));
                }
            };

        if parts.is_empty() {
            let msg = format!("Empty command: {}", cmd_str);
            return Err(bad_request(msg));
        }

        // 检查黑名单
        if let Some(reason) = check_blacklist(&parts) {
            let response = (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": format!("Command blocked: {}", reason),
                    "success": false,
                    "blocked_command": cmd_str
                })),
            )
                .into_response();
            return Err(response);
        }

        parsed_commands.push(parts);
    }

    Ok(parsed_commands)
}

/// 检查命令是否在黑名单中
///
/// # 参数
/// * `cmd_parts` - 切割后的命令部分（第一个是 executable，其余是参数）
///
/// # 返回
/// 如果命中黑名单，返回 Some(原因)；否则返回 None
fn check_blacklist(cmd_parts: &[String]) -> Option<String> {
    if cmd_parts.is_empty() {
        return Some("Empty command".to_string());
    }

    // 检查可执行文件名
    let exe_lower = cmd_parts[0].to_lowercase();
    let exe_filename = std::path::Path::new(&exe_lower)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| exe_lower.clone());

    for &blocked in COMMAND_BLACKLIST {
        // 跳过参数模式的黑名单项（以 - 或 / 开头）
        if blocked.starts_with('-') || blocked.starts_with('/') {
            continue;
        }
        if exe_filename == blocked.to_lowercase() {
            return Some(format!("Blocked executable: {}", cmd_parts[0]));
        }
    }

    // 检查所有参数部分
    for arg in &cmd_parts[1..] {
        let arg_lower = arg.to_lowercase();

        // 检查参数中是否包含危险可执行文件
        for &pattern in DANGEROUS_ARG_PATTERNS {
            if arg_lower.contains(&pattern.to_lowercase()) {
                return Some(format!("Blocked dangerous pattern in args: {}", pattern));
            }
        }

        // 检查参数中是否包含危险标志
        for &blocked in COMMAND_BLACKLIST {
            if blocked.starts_with('-') || blocked.starts_with('/') {
                if arg_lower == blocked.to_lowercase() {
                    return Some(format!("Blocked dangerous argument: {}", blocked));
                }
            }
        }
    }

    None
}

/// 执行解析后的命令列表，并返回统一的 HTTP 响应
///
/// # 参数
/// * `parsed_commands` - 已解析并验证的命令参数列表（每个命令为 `Vec<String>`）
///
/// # 返回
/// 包含执行结果的 JSON 响应（成功、部分成功或全失败）
pub fn execute_commands(parsed_commands: &[Vec<String>]) -> Response {
    let mut results = Vec::new();
    let mut has_error = false;

    for parts in parsed_commands {
        let exe = &parts[0];
        let args = &parts[1..];

        match Command::new(exe).args(args).spawn() {
            Ok(_) => {
                results.push(json!({
                    "command": parts.join(" "),
                    "success": true
                }));
            }
            Err(e) => {
                has_error = true;
                results.push(json!({
                    "command": parts.join(" "),
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    if has_error {
        (
            StatusCode::PARTIAL_CONTENT,
            Json(json!({
                "success": false,
                "partial": true,
                "results": results
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "results": results
            })),
        )
            .into_response()
    }
}
