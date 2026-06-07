// src-tauri/src/common/fs_ops.rs

use std::fs;
use std::path::Path;

#[tauri::command]
pub fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    let path_obj = Path::new(&path);
    if path_obj.exists() {
        return Err("文件已存在".to_string());
    }
    fs::write(&path, "").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_entry(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_entry(source_path: String, destination_path: String) -> Result<(), String> {
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
pub fn delete_entry(path: String) -> Result<(), String> {
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

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), target_path)?;
        }
    }

    Ok(())
}
