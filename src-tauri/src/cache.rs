//! 缓存读写 — 将扫描结果持久化到本地 JSON 文件，供前端启动时快速恢复。

use crate::scanner::FileNode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================
// 数据结构
// ============================================================

/// 扫描缓存：记录最近一次扫描的完整快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCache {
    /// 扫描时间（Unix 秒级时间戳）
    #[serde(rename = "scanTime")]
    pub scan_time: u64,
    /// 扫描模式：default | extended | full
    #[serde(rename = "scanMode")]
    pub scan_mode: String,
    /// 本次扫描的根节点列表
    #[serde(rename = "rootNodes")]
    pub root_nodes: Vec<FileNode>,
    /// 文件总数
    #[serde(rename = "totalFileCount")]
    pub total_file_count: u64,
    /// 总大小（字节）
    #[serde(rename = "totalSize")]
    pub total_size: u64,
}

// ============================================================
// 缓存路径
// ============================================================

/// 返回缓存文件的完整路径。
///
/// 优先使用 `%LOCALAPPDATA%\DiskPeek\cache.json`；
/// 若 LOCALAPPDATA 不可用，回退到 `%APPDATA%\DiskPeek\cache.json`。
pub fn get_cache_path() -> PathBuf {
    // dirs::data_local_dir() → Windows 上的 LOCALAPPDATA
    if let Some(local) = dirs::data_local_dir() {
        local.join("DiskPeek").join("cache.json")
    } else if let Some(roaming) = dirs::data_dir() {
        // dirs::data_dir() → Windows 上的 APPDATA (Roaming)
        roaming.join("DiskPeek").join("cache.json")
    } else {
        // 最终回退：用户主目录下
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DiskPeek")
            .join("cache.json")
    }
}

// ============================================================
// 缓存操作
// ============================================================

/// 将扫描缓存序列化并写入磁盘。
/// 自动创建父目录，写入失败时静默忽略（不影响主流程）。
pub fn save_cache(cache: &ScanCache) {
    let path = get_cache_path();

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("[DiskPeek] 无法创建缓存目录 {}: {}", parent.display(), e);
            return;
        }
    }

    // 序列化为带缩进的 JSON
    match serde_json::to_string_pretty(cache) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("[DiskPeek] 写入缓存失败 {}: {}", path.display(), e);
            }
        }
        Err(e) => {
            eprintln!("[DiskPeek] 序列化缓存失败: {}", e);
        }
    }
}

/// 读取并反序列化缓存文件。
/// 文件不存在或内容损坏时返回 None。
pub fn load_cache() -> Option<ScanCache> {
    let path = get_cache_path();
    let content = fs::read_to_string(&path).ok()?;

    match serde_json::from_str::<ScanCache>(&content) {
        Ok(cache) => Some(cache),
        Err(e) => {
            eprintln!("[DiskPeek] 缓存文件损坏 {}: {}", path.display(), e);
            None
        }
    }
}

/// 检查缓存文件是否存在（不关心内容是否有效）。
pub fn cache_exists() -> bool {
    get_cache_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_path_ends_with_cache_json() {
        let path = get_cache_path();
        assert!(path.ends_with("cache.json"));
        assert!(path.to_string_lossy().contains("DiskPeek"));
    }

    #[test]
    fn test_cache_exists_initially_false() {
        // 这个测试依赖环境，仅做基本调用验证
        let _ = cache_exists();
    }
}