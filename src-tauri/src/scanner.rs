//! 文件扫描引擎 — 遍历目录树，构建 FileNode 并按目录粒度增量推送扫描事件。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use walkdir::WalkDir;

// ============================================================
// 数据结构
// ============================================================

/// 文件树节点：代表一个文件或目录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    /// 基于路径 SHA256 前 16 位 hex 生成的唯一标识
    pub id: String,
    /// 文件或目录名
    pub name: String,
    /// 完整绝对路径
    pub path: String,
    /// 是否为目录
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    /// 文件大小（字节），目录为 0
    pub size: u64,
    /// 扩展名（小写，不含点号），目录为空串
    pub extension: String,
    /// 文件类型分类：VIDEO | IMAGE | DOCUMENT | ARCHIVE | OTHER；目录为 None
    #[serde(rename = "fileType")]
    pub file_type: Option<String>,
    /// 创建时间（Unix 秒级时间戳）
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    /// 最后修改时间（Unix 秒级时间戳）
    #[serde(rename = "modifiedAt")]
    pub modified_at: u64,
    /// 最后访问时间（Unix 秒级时间戳）
    #[serde(rename = "accessedAt")]
    pub accessed_at: u64,
    /// 子节点列表（仅目录有值）
    pub children: Vec<FileNode>,
    /// 父目录的 id，根节点为 None
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    /// 该节点所在目录是否因权限不足而跳过
    #[serde(rename = "permissionDenied")]
    pub permission_denied: bool,
}

/// 扫描事件：每完成一个目录的扫描后通过通道发送。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEvent {
    /// 当前扫描的根路径
    #[serde(rename = "rootPath")]
    pub root_path: String,
    /// 该目录及其直接子文件/子目录构成的节点树
    pub nodes: Vec<FileNode>,
}

// ============================================================
// 公开 API
// ============================================================

/// 返回默认扫描根目录列表：
/// Desktop、Documents、Downloads、Pictures、Videos、Music，以及 AppData（排除 Local/Temp 下的缓存）。
pub fn get_default_scan_roots() -> Vec<String> {
    let mut roots = Vec::new();

    // 逐个收集常用用户目录
    if let Some(p) = dirs::desktop_dir() {
        roots.push(p.to_string_lossy().to_string());
    }
    if let Some(p) = dirs::document_dir() {
        roots.push(p.to_string_lossy().to_string());
    }
    if let Some(p) = dirs::download_dir() {
        roots.push(p.to_string_lossy().to_string());
    }
    if let Some(p) = dirs::picture_dir() {
        roots.push(p.to_string_lossy().to_string());
    }
    if let Some(p) = dirs::video_dir() {
        roots.push(p.to_string_lossy().to_string());
    }
    if let Some(p) = dirs::audio_dir() {
        roots.push(p.to_string_lossy().to_string());
    }

    // AppData（Roaming），排除 Local/Temp 下的缓存
    if let Some(p) = dirs::data_dir() {
        roots.push(p.to_string_lossy().to_string());
    }

    roots
}

/// 扫描指定根目录，每完成一个目录的扫描即通过 tx 发送 ScanEvent。
///
/// - 跳过 junction / 符号链接（follow_links = false 且检测 symlink）
/// - 跳过已知缓存目录（node_modules、.git、__pycache__、target 等）
/// - 权限不足时静默跳过，并在对应节点标记 permission_denied
pub fn scan_directory(root_path: String, tx: Sender<ScanEvent>) {
    // 使用 walkdir 遍历，follow_links(false) 避免进入 junction
    let walker = WalkDir::new(&root_path)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();

    // 对每个目录：读取其直接子项，构建 FileNode 列表，发送 ScanEvent
    // - 跳过符号链接和缓存目录
    // - walkdir 会按深度优先遍历，每个目录（含根目录）都会被作为 entry 访问一次

    for entry in walker {
        match entry {
            Ok(e) => {
                let path = e.path();

                // 跳过符号链接 / junction
                if e.file_type().is_symlink() {
                    continue;
                }

                // 仅处理目录：读取其直接子项并发送事件
                if e.file_type().is_dir() {
                    let dir_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    // 跳过已知缓存目录
                    if is_cache_dir(&dir_name) {
                        continue;
                    }

                    // 收集该目录的直接子项
                    let children = match collect_children(path) {
                        Ok(c) => c,
                        Err(_) => {
                            // 权限不足：发送一个标记 permission_denied 的空事件
                            let mut denied_node = build_file_node(path, None);
                            denied_node.permission_denied = true;
                            let _ = tx.send(ScanEvent {
                                root_path: root_path.clone(),
                                nodes: vec![denied_node],
                            });
                            continue;
                        }
                    };

                    let parent_id = generate_id(&e.path().to_string_lossy());
                    // 构建当前目录节点，附带子项
                    let mut dir_node = build_file_node(path, None);
                    dir_node.children = children
                        .into_iter()
                        .map(|child_path| {
                            let mut node = build_file_node(&child_path, Some(parent_id.clone()));
                            // 如果子项也是目录，递归时 children 留空（由后续 entry 单独发送）
                            if child_path.is_dir() {
                                node.children = Vec::new();
                            }
                            node
                        })
                        .collect();

                    let _ = tx.send(ScanEvent {
                        root_path: root_path.clone(),
                        nodes: vec![dir_node],
                    });
                }
            }
            Err(_err) => {
                // walkdir 内部权限不足等错误：静默跳过
                // 通过 io::Error 提取路径并发送 permission_denied 事件
                if let Some(denied_path) = _err.path() {
                    let mut denied_node = build_file_node(denied_path, None);
                    denied_node.permission_denied = true;
                    let _ = tx.send(ScanEvent {
                        root_path: root_path.clone(),
                        nodes: vec![denied_node],
                    });
                }
            }
        }
    }
}

// ============================================================
// 内部辅助函数
// ============================================================

/// 根据扩展名分类文件类型。
/// 返回 Option<String>：VIDEO | IMAGE | DOCUMENT | ARCHIVE | OTHER；扩展名为空返回 None。
pub fn classify_file_type(ext: &str) -> Option<String> {
    if ext.is_empty() {
        return None;
    }

    let ext_lower = ext.to_lowercase();
    let category = match ext_lower.as_str() {
        // 视频
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
        | "3gp" | "rmvb" | "ts" => "VIDEO",
        // 图片
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        | "heic" | "raw" | "cr2" | "nef" => "IMAGE",
        // 文档
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "csv"
        | "json" | "xml" | "html" | "htm" | "rtf" | "odt" | "ods" | "odp" | "epub" | "mobi"
        | "log" | "yaml" | "yml" | "toml" | "ini" | "cfg" => "DOCUMENT",
        // 压缩包
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "lz" | "lz4" | "zst" | "iso"
        | "cab" | "arj" => "ARCHIVE",
        // 其他
        _ => "OTHER",
    };

    Some(category.to_string())
}

/// 判断目录名是否为已知缓存目录。
fn is_cache_dir(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "node_modules"
            | ".git"
            | "__pycache__"
            | "target"
            | "npm-cache"
            | "pip-cache"
            | ".cache"
            | ".npm"
            | ".yarn"
            | "vendor"
            | "bower_components"
            | ".next"
            | ".nuxt"
            | "dist"
            | "build"
    )
}

/// 基于路径的 SHA256 前 16 位 hex 生成唯一 id。
fn generate_id(path_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // 取前 8 字节 = 16 位 hex
}

/// 将路径的元数据填充到 FileNode 中。
fn build_file_node(path: &Path, parent_id: Option<String>) -> FileNode {
    let is_directory = path.is_dir();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let path_str = path.to_string_lossy().to_string();
    let id = generate_id(&path_str);

    let (size, extension, file_type) = if is_directory {
        (0, String::new(), None)
    } else {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ftype = classify_file_type(&ext);
        (0, ext, ftype) // 文件大小稍后填充
    };

    let (actual_size, created_at, modified_at, accessed_at) =
        match fs::metadata(path) {
            Ok(meta) => {
                let size_val = if is_directory { 0 } else { meta.len() };
                let created = meta
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let accessed = meta
                    .accessed()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (size_val, created, modified, accessed)
            }
            Err(_) => (0, 0, 0, 0),
        };

    FileNode {
        id,
        name,
        path: path_str,
        is_directory,
        size: actual_size,
        extension,
        file_type,
        created_at,
        modified_at,
        accessed_at,
        children: Vec::new(),
        parent_id,
        permission_denied: false,
    }
}

/// 收集目录下的直接子项路径列表（仅一层）。
fn collect_children(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut children = Vec::new();
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        match entry {
            Ok(e) => {
                let child_path = e.path();
                // 排除符号链接
                if e.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    continue;
                }
                children.push(child_path);
            }
            Err(_) => {
                // 单个子项读取失败，跳过
                continue;
            }
        }
    }
    // 按名称排序保证稳定输出
    children.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });
    Ok(children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_video() {
        assert_eq!(classify_file_type("mp4"), Some("VIDEO".into()));
        assert_eq!(classify_file_type("MKV"), Some("VIDEO".into()));
    }

    #[test]
    fn test_classify_image() {
        assert_eq!(classify_file_type("png"), Some("IMAGE".into()));
    }

    #[test]
    fn test_classify_document() {
        assert_eq!(classify_file_type("pdf"), Some("DOCUMENT".into()));
        assert_eq!(classify_file_type("xlsx"), Some("DOCUMENT".into()));
    }

    #[test]
    fn test_classify_archive() {
        assert_eq!(classify_file_type("zip"), Some("ARCHIVE".into()));
    }

    #[test]
    fn test_classify_other() {
        assert_eq!(classify_file_type("exe"), Some("OTHER".into()));
    }

    #[test]
    fn test_classify_empty() {
        assert_eq!(classify_file_type(""), None);
    }

    #[test]
    fn test_is_cache_dir() {
        assert!(is_cache_dir("node_modules"));
        assert!(is_cache_dir(".git"));
        assert!(is_cache_dir("__pycache__"));
        assert!(!is_cache_dir("my_project"));
    }

    #[test]
    fn test_generate_id_is_16_hex() {
        let id = generate_id("C:\\Users\\test\\file.txt");
        assert_eq!(id.len(), 16);
        // 应该全是 hex 字符
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}