// src-tauri/src/common/fs_utils.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

/// 定义文件节点的结构，用于前端文件树渲染
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,                    // 显示的文件名 (e.g. "st01.decl")
    pub path: String,                    // 绝对路径 (e.g. "D:/Project/st01.decl")
    pub is_dir: bool,                    // 是否为文件夹
    pub size: Option<u64>,               // 文件大小（字节），目录为 None
    pub extension: Option<String>,       // 文件后缀 (e.g. "decl"), 用于前端判断图标
    pub category: FileCategory,          // 文件分类 (用于逻辑判断)
    pub children: Option<Vec<FileNode>>, // 子节点 (仅文件夹有)
    #[serde(rename = "isLeaf")]
    pub is_leaf: bool,                   // NTree 需要 isLeaf 判断是否可展开
}

/// 文件分类枚举，帮助前端快速判断如何处理该文件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")] // 输出 json 为 "sourceScript", "binaryScript" 等
pub enum FileCategory {
    SourceScript,    // 我们自定义的中间格式 (.decl, .dmsg, .dstd) -> 用编辑器打开
    BinaryScript,    // 原始游戏脚本 (.ecl, .msg, .std) -> 提示需要反编译
    Archive,         // 包文件 (.dat)
    Image,           // 图片 (.png, .jpg) -> 用预览器打开
    AssetDefinition, // 动画定义 (.anm, .danm) -> 特殊编辑器
    Directory,       // 文件夹
    Unknown,         // 其他
}

/// 扫描目录并返回一层文件树（目录节点的 children 为 None，前端按需加载）
pub fn get_file_tree<P: AsRef<Path>>(root_path: P) -> Result<Vec<FileNode>> {
    list_dir_shallow(root_path.as_ref())
}

/// 按需加载：返回某个目录的直接子节点列表
pub fn get_dir_children<P: AsRef<Path>>(dir_path: P) -> Result<Vec<FileNode>> {
    list_dir_shallow(dir_path.as_ref())
}

/// 内部：浅层列出目录内容（只读一层）
fn list_dir_shallow(dir_path: &Path) -> Result<Vec<FileNode>> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(vec![]);
    }

    let mut nodes = Vec::new();
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();

        // 过滤以 . 开头的隐藏文件/文件夹
        if name.starts_with('.') {
            continue;
        }

        let extension = path
            .extension()
            .map(|os_str| os_str.to_string_lossy().to_string().to_lowercase());
        let size = if is_dir {
            None
        } else {
            entry.metadata().ok().map(|metadata| metadata.len())
        };

        let category = if is_dir {
            FileCategory::Directory
        } else {
            determine_category(extension.as_deref())
        };

        // 目录标记为非叶子（前端可展开），除非是空目录
        let is_leaf = if is_dir {
            dir_is_empty(&path)
        } else {
            true
        };

        nodes.push(FileNode {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            extension,
            category,
            children: None, // 浅层模式：不预加载子节点
            is_leaf,
        });
    }

    sort_nodes(&mut nodes);
    Ok(nodes)
}

/// 检查目录是否为空（忽略隐藏文件）
fn dir_is_empty(dir_path: &Path) -> bool {
    match fs::read_dir(dir_path) {
        Ok(mut entries) => !entries.any(|e| {
            e.ok()
                .map(|e| {
                    !e.file_name()
                        .to_string_lossy()
                        .starts_with('.')
                })
                .unwrap_or(false)
        }),
        Err(_) => true,
    }
}

/// 排序：文件夹优先，然后按字母顺序
fn sort_nodes(nodes: &mut Vec<FileNode>) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
}

/// 辅助函数：根据后缀判断文件分类
fn determine_category(extension: Option<&str>) -> FileCategory {
    match extension {
        Some("decl") | Some("dmsg") | Some("dstd") | Some("danm") => FileCategory::SourceScript,
        Some("ecl") | Some("msg") | Some("std") => FileCategory::BinaryScript,
        Some("anm") => FileCategory::AssetDefinition,
        Some("dat") => FileCategory::Archive,
        Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") => FileCategory::Image,
        _ => FileCategory::Unknown,
    }
}
