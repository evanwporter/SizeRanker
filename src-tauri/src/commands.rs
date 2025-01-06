use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct FileItem {
    path: String,
    name: String,
    size_bytes: u64,
    is_dir: bool,
    human_readable_size: String,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn get_size(path: &PathBuf) -> u64 {
    if path.is_file() {
        fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
    } else if path.is_dir() {
        let mut size = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                size += get_size(&entry_path);
            }
        }
        size
    } else {
        0
    }
}

#[tauri::command]
pub fn scan_directory(path: String) -> Result<Vec<FileItem>, String> {
    let dir_path = PathBuf::from(&path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", dir_path.display()));
    }

    let dir_path = dir_path
        .canonicalize()
        .map_err(|_| format!("Failed to resolve path: {}", dir_path.display()))?;

    let mut items = Vec::new();

    if let Some(parent_path) = dir_path.parent() {
        items.push(FileItem {
            path: parent_path.to_string_lossy().to_string(),
            name: "..".to_string(),
            size_bytes: 0,
            human_readable_size: "-".to_string(),
            is_dir: true,
        });
    }

    let mut directory_items = Vec::new();
    for entry in fs::read_dir(&dir_path).map_err(|_| "Failed to read directory".to_string())? {
        let entry = entry.map_err(|_| "Failed to read entry".to_string())?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|_| "Failed to get metadata".to_string())?;

        directory_items.push(FileItem {
            path: path.to_string_lossy().to_string(),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            size_bytes: get_size(&path),
            human_readable_size: format_size(get_size(&path)),
            is_dir: metadata.is_dir(),
        });
    }

    directory_items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    items.extend(directory_items);

    Ok(items)
}

#[tauri::command]
pub fn get_executable_directory() -> Result<String, String> {
    std::env::current_exe()
        .map(|exe_path| {
            exe_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
                .to_string()
        })
        .map_err(|_| "Failed to get executable directory".to_string())
}

#[tauri::command]
pub fn delete_files(paths: Vec<String>) -> Result<(), String> {
    for path_str in paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|_| format!("Failed to delete directory: {}", path.display()))?;
            } else {
                fs::remove_file(&path)
                    .map_err(|_| format!("Failed to delete file: {}", path.display()))?;
            }
        } else {
            return Err(format!("Path does not exist: {}", path.display()));
        }
    }
    Ok(())
}
