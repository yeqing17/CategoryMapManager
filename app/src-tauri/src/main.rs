#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

const PORTAL_PREFIX: &str = "portal_frag_";
const TEMPLATE_VALUE: &str =
    "com.ipanel.join.gw_ui_sdk.GwPortalFragment|intent://?es_tabId={id}&es_title=&es_focusStartColor=&es_focusEndColor=&es_focusImg=";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingEntry {
    local_id: String,
    gw_id: Option<String>,
    raw_value: String,
    same_id: bool,
    /// 状态：normal-正常, duplicate_local-本地ID重复, duplicate_gw-国网ID重复
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileMapping {
    file_path: String,
    mappings: Vec<MappingEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    files: Vec<FileMapping>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupResult {
    backup_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkippedFile {
    file_path: String,
    reason: String,
    duplicate_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BulkInsertResult {
    updated_files: Vec<String>,
    skipped_files: Vec<SkippedFile>,
    backup_dir: Option<String>,
}

/// 操作类型枚举
enum OperationType {
    BulkInsert,
    Import,
    BatchDelete,
    SingleDelete,
}

/// 删除的映射项信息
struct DeletedMapping {
    file_path: String,
    local_id: String,
    gw_id: Option<String>,
}

/// 新增的映射项信息
struct AddedMapping {
    file_path: String,
    local_id: String,
    gw_id: String,
}

/// 写入操作日志
fn write_operation_log(
    target_dir: &Path,
    operation_type: OperationType,
    updated_files: &[String],
    skipped_files: &[SkippedFile],
    backup_dir: Option<&String>,
    additional_info: Option<&str>,
    deleted_mappings: Option<&[DeletedMapping]>,
    added_mappings: Option<&[AddedMapping]>,
) -> Result<(), String> {
    let timestamp = Local::now();
    let log_filename = format!("operation_{}.log", timestamp.format("%Y%m%d-%H%M%S"));
    let log_path = target_dir.join(&log_filename);

    let mut log_content = String::new();
    
    // 写入日志头
    let separator = "=".repeat(80);
    log_content.push_str(&separator);
    log_content.push_str(&format!("\n操作日志 - {}\n", timestamp.format("%Y-%m-%d %H:%M:%S")));
    log_content.push_str(&format!("{}\n", separator));
    
    // 操作类型
    let op_type_str = match operation_type {
        OperationType::BulkInsert => "批量新增映射",
        OperationType::Import => "导入映射（替换模式）",
        OperationType::BatchDelete => "批量删除映射",
        OperationType::SingleDelete => "单个删除映射",
    };
    log_content.push_str(&format!("\n操作类型: {}\n", op_type_str));
    
    // 备份路径
    if let Some(backup_path) = backup_dir {
        log_content.push_str(&format!("备份路径: {}\n", backup_path));
    } else {
        log_content.push_str("备份路径: 无（未创建备份）\n");
    }
    
    // 附加信息
    if let Some(info) = additional_info {
        log_content.push_str(&format!("附加信息: {}\n", info));
    }
    
    // 新增的映射详情（仅对新增操作）
    if let Some(added) = added_mappings {
        if !added.is_empty() {
            log_content.push_str(&format!("\n新增的映射详情 ({} 条):\n", added.len()));
            // 按文件路径分组显示
            let mut file_groups: std::collections::HashMap<String, Vec<&AddedMapping>> = std::collections::HashMap::new();
            for mapping in added {
                file_groups.entry(mapping.file_path.clone()).or_insert_with(Vec::new).push(mapping);
            }
            
            for (file_path, mappings) in file_groups {
                log_content.push_str(&format!("  {}\n", file_path));
                for mapping in mappings {
                    log_content.push_str(&format!("    - 本地栏目ID: {} | 国网栏目ID: {}\n", mapping.local_id, mapping.gw_id));
                }
            }
        }
    }
    
    // 删除的映射详情（仅对删除操作）
    if let Some(deleted) = deleted_mappings {
        if !deleted.is_empty() {
            log_content.push_str(&format!("\n删除的映射详情 ({} 条):\n", deleted.len()));
            // 按文件路径分组显示
            let mut file_groups: std::collections::HashMap<String, Vec<&DeletedMapping>> = std::collections::HashMap::new();
            for mapping in deleted {
                file_groups.entry(mapping.file_path.clone()).or_insert_with(Vec::new).push(mapping);
            }
            
            for (file_path, mappings) in file_groups {
                log_content.push_str(&format!("  {}\n", file_path));
                for mapping in mappings {
                    if let Some(gw_id) = &mapping.gw_id {
                        log_content.push_str(&format!("    - 本地栏目ID: {} | 国网栏目ID: {}\n", mapping.local_id, gw_id));
                    } else {
                        log_content.push_str(&format!("    - 本地栏目ID: {} | 国网栏目ID: (无)\n", mapping.local_id));
                    }
                }
            }
        }
    }
    
    // 成功处理的文件
    log_content.push_str(&format!("\n成功处理的文件 ({} 个):\n", updated_files.len()));
    if updated_files.is_empty() {
        log_content.push_str("  无\n");
    } else {
        for file in updated_files {
            log_content.push_str(&format!("  - {}\n", file));
        }
    }
    
    // 跳过的文件
    if !skipped_files.is_empty() {
        log_content.push_str(&format!("\n跳过的文件 ({} 个):\n", skipped_files.len()));
        for skipped in skipped_files {
            log_content.push_str(&format!("  - {}\n", skipped.file_path));
            log_content.push_str(&format!("    原因: {}\n", skipped.reason));
            if !skipped.duplicate_ids.is_empty() {
                log_content.push_str(&format!("    相关ID: {}\n", skipped.duplicate_ids.join("、")));
            }
        }
    }
    
    // 日志尾
    let separator = "=".repeat(80);
    log_content.push_str(&format!("\n{}\n", separator));
    log_content.push_str("\n");
    
    // 写入文件
    let mut file = fs::File::create(&log_path).map_err(|e| format!("创建日志文件失败: {}", e))?;
    file.write_all(log_content.as_bytes())
        .map_err(|e| format!("写入日志文件失败: {}", e))?;
    
    Ok(())
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MappingInput {
    local_id: String,
    gw_id: String,
}

#[tauri::command]
fn scan_theme_files(target_dir: String) -> Result<ScanResult, String> {
    let dir = PathBuf::from(&target_dir);
    let files = collect_theme_files(&dir)?;

    let mut results = Vec::with_capacity(files.len());
    for file in files {
        let raw = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let mappings = parse_mappings(&raw)?;
        results.push(FileMapping {
            file_path: file.to_string_lossy().into_owned(),
            mappings,
        });
    }

    Ok(ScanResult { files: results })
}

#[tauri::command]
fn backup_theme_files(target_dir: String) -> Result<BackupResult, String> {
    let dir = PathBuf::from(&target_dir);
    let files = collect_theme_files(&dir)?;

    if files.is_empty() {
        return Err("当前目录下未找到 theme*.json 文件".into());
    }

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_dir = dir.join("backups").join(timestamp);
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

    for file in files {
        if let Some(name) = file.file_name() {
            let target = backup_dir.join(name);
            fs::copy(&file, target).map_err(|err| err.to_string())?;
        }
    }

    Ok(BackupResult {
        backup_dir: backup_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
fn bulk_insert_mappings(target_dir: String, entries: Vec<MappingInput>) -> Result<BulkInsertResult, String> {
    if entries.is_empty() {
        return Err("请至少输入一条映射关系。".into());
    }

    let dir = PathBuf::from(&target_dir);
    let files = collect_theme_files(&dir)?;
    let mut updated_files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut files_to_update: Vec<(PathBuf, Vec<MappingInput>)> = Vec::new();

    // 先检查哪些文件需要更新
    for file in &files {
        let raw = fs::read_to_string(file).map_err(|err| err.to_string())?;
        let parsed = parse_mappings(&raw)?;
        let existing: HashSet<String> = parsed.iter().map(|item| item.local_id.clone()).collect();
        
        // 找出重复的ID和需要添加的ID
        let mut duplicate_ids = Vec::new();
        let mut pending = Vec::new();
        
        for entry in &entries {
            if existing.contains(&entry.local_id) {
                duplicate_ids.push(entry.local_id.clone());
            } else {
                pending.push(entry.clone());
            }
        }

        if pending.is_empty() {
            skipped_files.push(SkippedFile {
                file_path: file.to_string_lossy().into_owned(),
                reason: if duplicate_ids.is_empty() {
                    "所有映射已存在".to_string()
                } else {
                    format!("本地栏目ID重复：{}", duplicate_ids.join("、"))
                },
                duplicate_ids,
            });
        } else {
            // 如果有部分重复，也要记录
            if !duplicate_ids.is_empty() {
                skipped_files.push(SkippedFile {
                    file_path: file.to_string_lossy().into_owned(),
                    reason: format!("部分ID重复（已跳过）：{}", duplicate_ids.join("、")),
                    duplicate_ids: duplicate_ids.clone(),
                });
            }
            files_to_update.push((file.clone(), pending));
        }
    }

    // 只有在有文件需要更新时才备份
    let mut backup_dir_path: Option<String> = None;
    if !files_to_update.is_empty() {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_dir = dir.join("backups").join(&timestamp);
        fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

        for file in &files {
            if let Some(name) = file.file_name() {
                let target = backup_dir.join(name);
                fs::copy(file, target).map_err(|err| err.to_string())?;
            }
        }
        backup_dir_path = Some(backup_dir.to_string_lossy().into_owned());
    }

    // 收集新增的映射详情（用于日志记录）
    let mut added_mappings: Vec<AddedMapping> = Vec::new();

    // 执行更新
    for (file, pending) in files_to_update {
        let file_path_str = file.to_string_lossy().into_owned();
        let raw = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let updated = insert_entries(&raw, &pending)?;
        fs::write(&file, updated).map_err(|err| err.to_string())?;
        updated_files.push(file_path_str.clone());
        
        // 记录新增的映射详情
        for entry in &pending {
            added_mappings.push(AddedMapping {
                file_path: file_path_str.clone(),
                local_id: entry.local_id.clone(),
                gw_id: entry.gw_id.clone(),
            });
        }
    }

    // 写入操作日志
    let entries_info = format!("新增 {} 条映射", entries.len());
    if let Err(e) = write_operation_log(
        &dir,
        OperationType::BulkInsert,
        &updated_files,
        &skipped_files,
        backup_dir_path.as_ref(),
        Some(&entries_info),
        None,
        Some(&added_mappings),
    ) {
        // 日志写入失败不影响主操作，只打印错误
        eprintln!("写入操作日志失败: {}", e);
    }

    Ok(BulkInsertResult {
        updated_files,
        skipped_files,
        backup_dir: backup_dir_path,
    })
}

#[tauri::command]
fn import_mappings(
    target_dir: String,
    mappings: std::collections::HashMap<String, String>,
) -> Result<BulkInsertResult, String> {
    if mappings.is_empty() {
        return Err("导入的映射为空".into());
    }

    let dir = PathBuf::from(&target_dir);
    let files = collect_theme_files(&dir)?;
    let mut updated_files = Vec::new();

    // 先备份
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_dir = dir.join("backups").join(timestamp);
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

    for file in &files {
        if let Some(name) = file.file_name() {
            let target = backup_dir.join(name);
            fs::copy(file, target).map_err(|err| err.to_string())?;
        }
    }

    // 对每个文件执行导入（替换模式）
    for file in files {
        let raw = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let updated = replace_mappings_in_file(&raw, &mappings)?;
        fs::write(&file, updated).map_err(|err| err.to_string())?;
        updated_files.push(file.to_string_lossy().into_owned());
    }

    // 写入操作日志
    let mappings_info = format!("导入 {} 条映射（替换模式）", mappings.len());
    if let Err(e) = write_operation_log(
        &dir,
        OperationType::Import,
        &updated_files,
        &[],
        Some(&backup_dir.to_string_lossy().into_owned()),
        Some(&mappings_info),
        None,
        None,
    ) {
        // 日志写入失败不影响主操作，只打印错误
        eprintln!("写入操作日志失败: {}", e);
    }

    Ok(BulkInsertResult {
        updated_files,
        skipped_files: Vec::new(),
        backup_dir: Some(backup_dir.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
fn delete_mapping(file_path: String, local_id: String) -> Result<Option<String>, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("文件不存在".into());
    }

    // 创建备份
    let file_dir = path.parent().ok_or("无法获取文件所在目录")?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_dir = file_dir.join("backups").join(&timestamp);
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

    if let Some(name) = path.file_name() {
        let target = backup_dir.join(name);
        fs::copy(&path, target).map_err(|err| err.to_string())?;
    }

    let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    
    // 先解析文件获取国网ID（用于日志记录）
    let parsed_mappings = parse_mappings(&raw).unwrap_or_default();
    let gw_id = parsed_mappings.iter()
        .find(|e| e.local_id == local_id)
        .and_then(|e| e.gw_id.clone());
    
    let updated = remove_mapping_from_file(&raw, &local_id)?;
    fs::write(&path, updated).map_err(|err| err.to_string())?;

    // 写入操作日志
    let delete_info = format!("删除本地栏目ID: {}", local_id);
    let deleted_mappings = vec![DeletedMapping {
        file_path: file_path.clone(),
        local_id: local_id.clone(),
        gw_id,
    }];
    
    if let Err(e) = write_operation_log(
        file_dir,
        OperationType::SingleDelete,
        &[file_path.clone()],
        &[],
        Some(&backup_dir.to_string_lossy().into_owned()),
        Some(&delete_info),
        Some(&deleted_mappings),
        None,
    ) {
        // 日志写入失败不影响主操作，只打印错误
        eprintln!("写入操作日志失败: {}", e);
    }

    Ok(Some(backup_dir.to_string_lossy().into_owned()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteMappingRequest {
    file_path: String,
    local_id: String,
}

#[tauri::command]
fn batch_delete_mappings(requests: Vec<DeleteMappingRequest>) -> Result<BulkInsertResult, String> {
    if requests.is_empty() {
        return Err("删除列表为空".into());
    }

    let mut updated_files = Vec::new();
    let mut skipped_files = Vec::new();

    // 按文件路径分组，提高效率
    let mut file_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for req in &requests {
        file_groups
            .entry(req.file_path.clone())
            .or_insert_with(Vec::new)
            .push(req.local_id.clone());
    }

    // 收集所有需要备份的文件路径
    let files_to_backup: Vec<PathBuf> = file_groups
        .keys()
        .map(|fp| PathBuf::from(fp))
        .filter(|p| p.exists())
        .collect();

    // 创建备份（如果有文件需要更新）
    let mut backup_dir_path: Option<String> = None;
    if !files_to_backup.is_empty() {
        // 找到所有文件的共同父目录（如果都在同一目录下）
        // 如果文件在不同目录，则使用第一个文件的目录
        let first_file = &files_to_backup[0];
        let file_dir = first_file.parent().ok_or("无法获取文件所在目录")?;
        
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_dir = file_dir.join("backups").join(&timestamp);
        fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

        // 备份所有涉及的文件
        for file_path in &files_to_backup {
            if let Some(name) = file_path.file_name() {
                let target = backup_dir.join(name);
                fs::copy(file_path, target).map_err(|err| err.to_string())?;
            }
        }
        backup_dir_path = Some(backup_dir.to_string_lossy().into_owned());
    }

    // 收集删除的映射详情（用于日志记录）
    let mut deleted_mappings: Vec<DeletedMapping> = Vec::new();

    // 对每个文件批量删除
    for (file_path, local_ids) in file_groups {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            skipped_files.push(SkippedFile {
                file_path: file_path.clone(),
                reason: "文件不存在".to_string(),
                duplicate_ids: local_ids,
            });
            continue;
        }

        let raw = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                skipped_files.push(SkippedFile {
                    file_path: file_path.clone(),
                    reason: format!("读取文件失败: {}", e),
                    duplicate_ids: local_ids,
                });
                continue;
            }
        };

        // 先解析文件获取映射信息（用于记录日志）
        let parsed_mappings = parse_mappings(&raw).unwrap_or_default();
        let mut mapping_map: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
        for entry in parsed_mappings {
            mapping_map.insert(entry.local_id, entry.gw_id);
        }

        // 逐个删除
        let mut current_content = raw;
        let mut successfully_deleted_ids = Vec::new();
        let mut failed_to_delete_ids = Vec::new();
        
        for local_id in &local_ids {
        // 在删除前记录映射信息
        let gw_id = mapping_map.get(local_id).cloned().flatten();
        
        match remove_mapping_from_file(&current_content, local_id) {
            Ok(updated) => {
                current_content = updated;
                successfully_deleted_ids.push(local_id.clone());
                // 记录成功删除的映射详情
                deleted_mappings.push(DeletedMapping {
                    file_path: file_path.clone(),
                    local_id: local_id.clone(),
                    gw_id,
                });
            }
                Err(_) => {
                    failed_to_delete_ids.push(local_id.clone());
                    // 记录错误但不中断处理
                }
            }
        }

        if !successfully_deleted_ids.is_empty() {
            if let Err(err) = fs::write(&path, current_content) {
                skipped_files.push(SkippedFile {
                    file_path: file_path.clone(),
                    reason: format!("写入文件失败: {}", err),
                    duplicate_ids: successfully_deleted_ids,
                });
            } else {
                updated_files.push(file_path.clone());
                // 如果有部分失败，记录跳过的文件
                if !failed_to_delete_ids.is_empty() {
                    skipped_files.push(SkippedFile {
                        file_path: file_path.clone(),
                        reason: format!("部分ID未找到或删除失败：{}", failed_to_delete_ids.join("、")),
                        duplicate_ids: failed_to_delete_ids,
                    });
                }
            }
        } else {
            // 所有ID都删除失败
            skipped_files.push(SkippedFile {
                file_path: file_path.clone(),
                reason: format!("所有ID删除失败：{}", failed_to_delete_ids.join("、")),
                duplicate_ids: failed_to_delete_ids,
            });
        }
    }

    // 写入操作日志
    // 确定目标目录（使用第一个文件的目录）
    let log_target_dir = if let Some(first_file) = files_to_backup.first() {
        first_file.parent()
            .map(|p| p.to_path_buf())
            .ok_or("无法获取文件所在目录")?
    } else if let Some(first_updated) = updated_files.first() {
        // 获取父目录路径并转换为 PathBuf 以拥有所有权
        PathBuf::from(first_updated)
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or("无法获取文件所在目录")?
    } else {
        // 如果没有成功处理的文件，跳过日志记录
        return Ok(BulkInsertResult {
            updated_files,
            skipped_files,
            backup_dir: backup_dir_path,
        });
    };

    let delete_info = format!("批量删除 {} 条映射", requests.len());
    if let Err(e) = write_operation_log(
        &log_target_dir,
        OperationType::BatchDelete,
        &updated_files,
        &skipped_files,
        backup_dir_path.as_ref(),
        Some(&delete_info),
        Some(&deleted_mappings),
        None,
    ) {
        // 日志写入失败不影响主操作，只打印错误
        eprintln!("写入操作日志失败: {}", e);
    }

    Ok(BulkInsertResult {
        updated_files,
        skipped_files,
        backup_dir: backup_dir_path,
    })
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_theme_files,
            backup_theme_files,
            bulk_insert_mappings,
            import_mappings,
            delete_mapping,
            batch_delete_mappings,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn collect_theme_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.exists() {
        return Err("目标目录不存在".into());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).min_depth(1).max_depth(1) {
        let entry = entry.map_err(|err| err.to_string())?;
        if entry.file_type().is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("theme") && name.ends_with(".json") {
                    files.push(entry.into_path());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn parse_mappings(raw: &str) -> Result<Vec<MappingEntry>, String> {
    // 直接从文本中查找所有 portal_frag_* 条目，而不是从JSON对象中获取
    // 因为JSON解析时重复的key会被覆盖，无法检测到重复
    let mut result = Vec::new();
    parse_portal_frag_from_text(raw, &mut result)?;
    
    // 检查重复：本地ID必须唯一，国网ID可以重复但需要提示
    let mut local_id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut gw_id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    // 统计出现次数
    for entry in &result {
        *local_id_counts.entry(entry.local_id.clone()).or_insert(0) += 1;
        if let Some(ref gw_id) = entry.gw_id {
            *gw_id_counts.entry(gw_id.clone()).or_insert(0) += 1;
        }
    }
    
    // 标记状态
    for entry in &mut result {
        let local_count = local_id_counts.get(&entry.local_id).copied().unwrap_or(0);
        let gw_count = entry.gw_id.as_ref()
            .and_then(|gw| gw_id_counts.get(gw).copied())
            .unwrap_or(0);
        
        if local_count > 1 {
            entry.status = "duplicate_local".to_string();
        } else if gw_count > 1 {
            entry.status = "duplicate_gw".to_string();
        } else {
            entry.status = "normal".to_string();
        }
    }
    
    Ok(result)
}

/// 从原始文本中直接解析所有 portal_frag_* 条目，支持检测重复的key
fn parse_portal_frag_from_text(raw: &str, acc: &mut Vec<MappingEntry>) -> Result<(), String> {
    // 先找到 sExtOptions 块的位置
    let (block_start, block_end) = find_ext_options_block(raw)?;
    let block_content = &raw[block_start..=block_end];
    
    // 使用逐字符解析，查找所有 "portal_frag_xxx":"value" 的模式，同时跳过注释
    let prefix = format!("\"{PORTAL_PREFIX}");
    let bytes = block_content.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    
    while i < bytes.len() {
        let ch = bytes[i];
        
        // 处理注释
        if in_line_comment {
            if ch == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        
        if in_block_comment {
            if ch == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        
        // 处理字符串
        if in_string {
            if escape {
                escape = false;
            } else if ch == b'\\' {
                escape = true;
            } else if ch == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        
        // 检查注释开始
        if ch == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                in_line_comment = true;
                i += 2;
                continue;
            }
            if bytes[i + 1] == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }
        
        // 检查字符串开始
        if ch == b'"' {
            in_string = true;
            // 检查是否是 portal_frag_ 开头的key
            if i + prefix.len() <= bytes.len() {
                // 使用 get() 方法安全地获取字符串切片，避免字符边界问题
                if let Some(candidate) = block_content.get(i..i + prefix.len()) {
                    if candidate == prefix {
                        // 找到了一个可能的 portal_frag_ 条目
                        if let Some(entry) = parse_portal_entry_at(block_content, i, bytes) {
                            acc.push(entry);
                        }
                    }
                }
            }
        }
        
        i += 1;
    }
    
    Ok(())
}

/// 在指定位置解析一个 portal_frag_ 条目
fn parse_portal_entry_at(
    content: &str,
    start: usize,
    bytes: &[u8],
) -> Option<MappingEntry> {
    // key 的开始引号在 start，跳过它
    let key_start = start + 1;
    let key_end = find_string_end(content, key_start, bytes)?;
    
    // 使用 get() 方法安全地获取字符串切片
    let full_key = content.get(key_start..key_end)?;
    
    if !full_key.starts_with(PORTAL_PREFIX) {
        return None;
    }
    
    // 跳过冒号和空白
    let mut value_start = key_end + 1;
    while value_start < bytes.len()
        && (bytes[value_start] == b':'
            || bytes[value_start].is_ascii_whitespace())
    {
        value_start += 1;
    }
    
    // 查找值的开始引号
    if value_start >= bytes.len() || bytes[value_start] != b'"' {
        return None;
    }
    
    value_start += 1; // 跳过引号
    let value_end = find_string_end(content, value_start, bytes)?;
    
    // 使用 get() 方法安全地获取字符串切片
    let raw_value = content.get(value_start..value_end)?;
    
    let local_id = full_key.trim_start_matches(PORTAL_PREFIX).to_string();
    let gw_id = extract_gw_id(raw_value);
    let same_id = gw_id.as_ref().map(|gw| gw == &local_id).unwrap_or(false);
    
    Some(MappingEntry {
        local_id,
        gw_id,
        raw_value: raw_value.to_string(),
        same_id,
        status: "normal".to_string(),
    })
}

/// 查找字符串的结束位置（考虑转义）
fn find_string_end(_content: &str, start: usize, bytes: &[u8]) -> Option<usize> {
    let mut i = start;
    let mut escaped = false;
    
    while i < bytes.len() {
        let ch = bytes[i];
        if escaped {
            escaped = false;
        } else if ch == b'\\' {
            escaped = true;
        } else if ch == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}


fn extract_gw_id(raw_value: &str) -> Option<String> {
    let marker = "es_tabId=";
    let start = raw_value.find(marker)? + marker.len();
    let remainder = &raw_value[start..];
    let mut end = remainder.len();
    for delimiter in ['&', '|', '"'] {
        if let Some(pos) = remainder.find(delimiter) {
            end = end.min(pos);
        }
    }
    let gw_id = remainder[..end].trim();
    if gw_id.is_empty() {
        None
    } else {
        Some(gw_id.to_string())
    }
}

fn insert_entries(raw: &str, entries: &[MappingInput]) -> Result<String, String> {
    let (block_start, block_end) = find_ext_options_block(raw)?;
    let line_ending = if raw.contains("\r\n") { "\r\n" } else { "\n" };
    let interior = &raw[block_start + 1..block_end];
    let has_existing = interior.trim().is_empty() == false;

    let base_indent = detect_base_indent(raw, block_start);
    let entry_indent = format!("{base_indent}  ");
    let before_closing = &raw[..block_end];
    let ws_start = trim_trailing_whitespace_start(before_closing);

    let mut insertion = String::new();
    insertion.push_str(line_ending);

    for (idx, entry) in entries.iter().enumerate() {
        insertion.push_str(&entry_indent);
        insertion.push_str(&format_entry(entry));
        if idx < entries.len() - 1 {
            insertion.push(',');
        }
        insertion.push_str(line_ending);
    }
    insertion.push_str(&base_indent);

    let mut updated = String::with_capacity(raw.len() + insertion.len());
    updated.push_str(&before_closing[..ws_start]);
    if has_existing {
        updated.push(',');
    }
    updated.push_str(&insertion);
    updated.push_str(&raw[block_end..]);
    Ok(updated)
}

fn detect_base_indent(content: &str, block_start: usize) -> String {
    let prefix = &content[..block_start];
    if let Some((_, line)) = prefix.rsplit_once('\n') {
        line.chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>()
    } else {
        String::new()
    }
}

fn format_entry(entry: &MappingInput) -> String {
    let value = TEMPLATE_VALUE.replace("{id}", &entry.gw_id);
    format!("\"{PORTAL_PREFIX}{key}\":\"{value}\"", key = entry.local_id)
}

/// 替换文件中的映射项（导入模式）
fn replace_mappings_in_file(
    raw: &str,
    mappings: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let (block_start, block_end) = find_ext_options_block(raw)?;
    let line_ending = if raw.contains("\r\n") { "\r\n" } else { "\n" };
    let base_indent = detect_base_indent(raw, block_start);
    let entry_indent = format!("{base_indent}  ");

    // 解析现有内容，移除所有 portal_frag_* 条目
    let interior = &raw[block_start + 1..block_end];
    let lines: Vec<&str> = interior.split('\n').collect();
    let mut filtered_lines = Vec::new();

    for line in lines.iter() {
        let trimmed = line.trim();
        
        // 跳过包含 portal_frag_ 的行（包括注释行）
        if trimmed.contains(&format!("\"{PORTAL_PREFIX}")) || 
           (trimmed.starts_with("//") && trimmed.contains("portal_frag_")) {
            continue;
        }
        
        // 保留非 portal_frag_ 的行
        filtered_lines.push(*line);
    }

    // 构建新的内容
    let mut new_content = String::new();
    
    // 添加过滤后的现有内容（如果有）
    if !filtered_lines.is_empty() {
        let filtered_text = filtered_lines.join("\n");
        let trimmed_filtered = filtered_text.trim();
        if !trimmed_filtered.is_empty() {
            new_content.push_str(&trimmed_filtered);
            if !trimmed_filtered.ends_with(',') {
                new_content.push(',');
            }
            new_content.push_str(line_ending);
        }
    }

    // 添加新的映射项
    let mut mapping_vec: Vec<_> = mappings.iter().collect();
    mapping_vec.sort_by_key(|(k, _)| *k);

    for (idx, (local_id, raw_value)) in mapping_vec.iter().enumerate() {
        new_content.push_str(&entry_indent);
        new_content.push_str(&format!("\"{PORTAL_PREFIX}{key}\":\"{value}\"", key = local_id, value = raw_value));
        if idx < mapping_vec.len() - 1 {
            new_content.push(',');
        }
        new_content.push_str(line_ending);
    }

    // 构建最终结果
    let mut result = String::with_capacity(raw.len() + new_content.len());
    result.push_str(&raw[..block_start + 1]);
    result.push_str(&new_content);
    result.push_str(&base_indent);
    result.push_str(&raw[block_end..]);
    
    Ok(result)
}

fn find_ext_options_block(content: &str) -> Result<(usize, usize), String> {
    let key = "\"sExtOptions\"";
    let key_index = content.find(key).ok_or("未找到 sExtOptions 段落")?;
    let mut idx = key_index + key.len();
    let bytes = content.as_bytes();
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b':' {
        return Err("sExtOptions 定义格式不正确".into());
    }
    idx += 1;
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'{' {
        return Err("sExtOptions 不是对象类型".into());
    }
    let mut i = idx;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut block_start = idx;

    while i < bytes.len() {
        let ch = bytes[i];
        if in_line_comment {
            if ch == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if ch == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_string {
            if escape {
                escape = false;
            } else if ch == b'\\' {
                escape = true;
            } else if ch == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if ch == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                in_line_comment = true;
                i += 2;
                continue;
            }
            if bytes[i + 1] == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }
        if ch == b'{' {
            if depth == 0 {
                block_start = i;
            }
            depth += 1;
        } else if ch == b'}' {
            depth -= 1;
            if depth == 0 {
                return Ok((block_start, i));
            }
        }
        i += 1;
    }
    Err("未能定位 sExtOptions 的结束位置".into())
}

fn trim_trailing_whitespace_start(content: &str) -> usize {
    let mut idx = content.len();
    while idx > 0 {
        let ch = content.as_bytes()[idx - 1];
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
            idx -= 1;
        } else {
            break;
        }
    }
    idx
}

/// 从文件中删除指定的映射项
fn remove_mapping_from_file(raw: &str, local_id: &str) -> Result<String, String> {
    let (block_start, block_end) = find_ext_options_block(raw)?;
    let line_ending = if raw.contains("\r\n") { "\r\n" } else { "\n" };
    let interior = &raw[block_start + 1..block_end];
    
    // 按行分割，过滤掉包含目标 local_id 的行
    let lines: Vec<&str> = interior.split('\n').collect();
    let mut filtered_lines = Vec::new();
    let target_key = format!("\"{PORTAL_PREFIX}{local_id}\"");
    let mut found_target = false;
    
    for (_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // 检查是否包含目标 key（包括注释行）
        if trimmed.contains(&target_key) || 
           (trimmed.starts_with("//") && trimmed.contains(&local_id)) {
            found_target = true;
            // 跳过这一行
            continue;
        }
        // 保留其他行
        filtered_lines.push(*line);
    }
    
    if !found_target {
        return Err(format!("未找到本地栏目ID: {}", local_id));
    }
    
    // 清理末尾多余的逗号和空行
    while let Some(last) = filtered_lines.last() {
        let trimmed = last.trim();
        if trimmed.is_empty() {
            filtered_lines.pop();
        } else {
            break;
        }
    }
    
    // 处理逗号：确保 JSON 格式正确
    // 移除所有行末尾的逗号，然后重新添加（除了最后一行）
    let mut cleaned_lines = Vec::new();
    for (idx, line) in filtered_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut cleaned = trimmed.trim_end_matches(',').trim_end().to_string();
        // 如果不是最后一行，且下一行不是空行，添加逗号
        if idx < filtered_lines.len() - 1 {
            let next_trimmed = filtered_lines[idx + 1].trim();
            if !next_trimmed.is_empty() {
                cleaned.push(',');
            }
        }
        // 恢复原始缩进
        let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
        cleaned_lines.push(format!("{}{}", indent, cleaned));
    }
    
    // 构建最终结果
    let filtered_text = cleaned_lines.join("\n");
    let mut result = String::with_capacity(raw.len());
    result.push_str(&raw[..block_start + 1]);
    if !filtered_text.trim().is_empty() {
        result.push_str(&filtered_text);
        result.push_str(line_ending);
    }
    result.push_str(&raw[block_end..]);
    
    Ok(result)
}
