//! DiskPeek Tauri 入口 — 注册 IPC 命令，桥接前端与扫描引擎。

mod cache;
mod scanner;

use cache::{cache_exists, get_cache_path, load_cache, save_cache, ScanCache};
use scanner::{get_default_scan_roots, scan_directory, FileNode};
use serde::Serialize;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;

// ============================================================
// 事件负载结构体（发送给前端）
// ============================================================

/// scan_progress 事件负载：增量推送单个目录的扫描结果。
#[derive(Debug, Clone, Serialize)]
struct ScanProgressPayload {
    #[serde(rename = "rootPath")]
    root_path: String,
    nodes: Vec<FileNode>,
}

/// scan_complete 事件负载：扫描全部完成后的汇总信息。
#[derive(Debug, Clone, Serialize)]
struct ScanCompletePayload {
    #[serde(rename = "totalFileCount")]
    total_file_count: u64,
    #[serde(rename = "totalSize")]
    total_size: u64,
    #[serde(rename = "scanTime")]
    scan_time: u64,
}

/// check_cache 返回值。
#[derive(Debug, Clone, Serialize)]
struct CheckCacheResult {
    exists: bool,
    #[serde(rename = "scanTime")]
    scan_time: u64,
}

// ============================================================
// Tauri 命令
// ============================================================

/// 启动文件扫描任务。
///
/// 参数 `mode`：扫描范围 — "default" | "extended" | "full"。
/// 当前版本中 default / extended / full 均使用默认扫描根目录；
/// 后续可扩展 extended 增加 C 盘根、full 增加所有盘符。
#[tauri::command(rename = "scan")]
fn scan_command(app_handle: tauri::AppHandle, mode: String) {
    // 根据模式确定扫描根目录列表
    let roots = match mode.as_str() {
        "default" => get_default_scan_roots(),
        "extended" | "full" => {
            // 扩展模式：默认根 + C 盘根
            let mut r = get_default_scan_roots();
            r.push("C:\\".to_string());
            r
        }
        _ => get_default_scan_roots(),
    };

    // 后台线程执行扫描
    thread::spawn(move || {
        let scan_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut total_file_count: u64 = 0;
        let mut total_size: u64 = 0;
        // 收集所有根节点（用于缓存）
        let mut root_nodes: Vec<FileNode> = Vec::new();

        for root in &roots {
            let (tx, rx) = mpsc::channel::<ScanEvent>();

            let root_clone = root.clone();
            let app_handle_clone = app_handle.clone();

            // 在子线程中扫描当前根目录
            let scan_thread = thread::spawn(move || {
                scan_directory(root_clone, tx);
            });

            // 主线程消费扫描事件，通过 Tauri 事件推送到前端
            for event in rx {
                // 统计文件数和大小
                count_files_and_size(&event.nodes, &mut total_file_count, &mut total_size);

                // 收集根节点（仅顶层）
                for node in &event.nodes {
                    if node.parent_id.is_none() {
                        root_nodes.push(node.clone());
                    }
                }

                let payload = ScanProgressPayload {
                    root_path: event.root_path.clone(),
                    nodes: event.nodes,
                };
                let _ = app_handle_clone.emit("scan_progress", payload);
            }

            // 等待扫描线程结束
            let _ = scan_thread.join();
        }

        // 扫描完成：写入缓存
        let cache = ScanCache {
            scan_time,
            scan_mode: mode.clone(),
            root_nodes,
            total_file_count,
            total_size,
        };
        save_cache(&cache);

        // 向 Tauri 注册 scan_path 变量，供前端获取缓存路径
        let _ = app_handle.emit("scan_complete", ScanCompletePayload {
            total_file_count,
            total_size,
            scan_time,
        });
    });
}

/// 检查缓存是否存在，返回存在标志与上次扫描时间。
#[tauri::command]
fn check_cache_command() -> CheckCacheResult {
    if cache_exists() {
        match load_cache() {
            Some(c) => CheckCacheResult {
                exists: true,
                scan_time: c.scan_time,
            },
            None => CheckCacheResult {
                exists: false,
                scan_time: 0,
            },
        }
    } else {
        CheckCacheResult {
            exists: false,
            scan_time: 0,
        }
    }
}

/// 加载完整缓存数据，返回给前端用于快速恢复。
/// 缓存路径通过事件 `cache_loaded` 一同下发。
#[tauri::command]
fn load_cache_command(app_handle: tauri::AppHandle) -> Option<ScanCache> {
    let cache = load_cache();
    if let Some(ref c) = cache {
        // 下发缓存路径供前端展示
        let _ = app_handle.emit(
            "cache_path",
            serde_json::json!({ "path": get_cache_path().to_string_lossy() }),
        );
        let _ = app_handle.emit("cache_loaded", c);
    }
    cache
}

/// 在 Windows 资源管理器中定位并高亮指定文件。
///
/// 调用 `explorer /select,<path>` 打开资源管理器并选中文件。
#[tauri::command(rename = "open_in_explorer")]
fn open_in_explorer_command(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .args(["/select,", &path])
        .spawn()
        .map_err(|e| format!("无法打开资源管理器: {}", e))?;
    Ok(())
}

// ============================================================
// 辅助函数
// ============================================================

/// 递归统计 FileNode 树中的文件数量和总大小。
fn count_files_and_size(nodes: &[FileNode], total_count: &mut u64, total_size: &mut u64) {
    for node in nodes {
        if !node.is_directory {
            *total_count += 1;
            *total_size += node.size;
        }
        if !node.children.is_empty() {
            count_files_and_size(&node.children, total_count, total_size);
        }
    }
}

// ============================================================
// 程序入口
// ============================================================

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            scan_command,
            check_cache_command,
            load_cache_command,
            open_in_explorer_command,
        ])
        .run(tauri::generate_context!())
        .expect("DiskPeek 启动失败");
}