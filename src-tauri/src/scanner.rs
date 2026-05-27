//! 文件扫描引擎 — 遍历目录树，构建 FileNode 并按目录粒度增量推送扫描事件。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use uuid::Uuid;
use walkdir::WalkDir;

// ============================================================
// 数据结构
// ============================================================

/// 文件树节点：代表一个文件或目录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    /// 基于路径 UUID v5 生成的唯一标识
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
    /// 文件类型分类：VIDEO | IMAGE | AUDIO | DOCUMENT | ARCHIVE | OTHER；目录为 None
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

/// 返回默认扫描根目录列表。
pub fn get_default_scan_roots() -> Vec<String> {
    let mut roots = Vec::new();

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
    if let Some(p) = dirs::data_dir() {
        roots.push(p.to_string_lossy().to_string());
    }

    roots
}

/// 扫描指定根目录，使用栈驱动的单次遍历算法。
///
/// - 消除了 `collect_children` 的冗余 `fs::read_dir` 调用
/// - 复用 WalkDir 已缓存的 metadata
/// - 跳过符号链接和已知缓存目录
/// - 权限不足时标记 permission_denied
pub fn scan_directory(root_path: &str, tx: &Sender<ScanEvent>) {
    let walker = WalkDir::new(root_path)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();

    // 栈: (目录路径, 目录节点, 收集的子节点列表, 深度)
    let mut stack: Vec<(std::path::PathBuf, FileNode, Vec<FileNode>, usize)> = Vec::new();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                if let Some(p) = err.path() {
                    let mut node = build_file_node(p, None, None);
                    node.permission_denied = true;
                    let _ = tx.send(ScanEvent {
                        root_path: root_path.to_string(),
                        nodes: vec![node],
                    });
                }
                continue;
            }
        };

        let path = entry.path();
        if entry.file_type().is_symlink() {
            continue;
        }

        let depth = entry.depth();
        let is_dir = entry.file_type().is_dir();

        // 深度下降 = 外层目录扫描完成，弹出栈并发送事件
        while stack.last().map_or(false, |(_, _, _, d)| *d >= depth) {
            let (_dir_path, mut dir_node, children, _) = stack.pop().unwrap();
            dir_node.children = children;
            let _ = tx.send(ScanEvent {
                root_path: root_path.to_string(),
                nodes: vec![dir_node.clone()],
            });
            // 将完成的目录节点追加到新栈顶的 children
            if let Some(top) = stack.last_mut() {
                top.2.push(dir_node);
            }
        }

        if is_dir {
            let dir_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if is_cache_dir(&dir_name) {
                continue;
            }
            let meta = entry.metadata().ok();
            let dir_node = build_file_node(path, None, meta.as_ref());
            stack.push((path.to_path_buf(), dir_node, Vec::new(), depth));
        } else {
            let parent_id = stack.last().map(|(_, node, _, _)| node.id.clone());
            let meta = entry.metadata().ok();
            let file_node = build_file_node(path, parent_id, meta.as_ref());
            if let Some(top) = stack.last_mut() {
                top.2.push(file_node);
            }
        }
    }

    // 遍历结束后，弹出所有剩余栈条目
    while let Some((_dir_path, mut dir_node, children, _)) = stack.pop() {
        dir_node.children = children;
        let _ = tx.send(ScanEvent {
            root_path: root_path.to_string(),
            nodes: vec![dir_node.clone()],
        });
        if let Some(top) = stack.last_mut() {
            top.2.push(dir_node);
        }
    }
}

// ============================================================
// 内部辅助函数
// ============================================================

/// UUID v5 命名空间常量
const DISKPEEK_NS: Uuid = Uuid::NAMESPACE_URL;

/// 基于路径的 UUID v5 生成唯一 id。
fn generate_id(path_str: &str) -> String {
    Uuid::new_v5(&DISKPEEK_NS, path_str.as_bytes()).to_string()
}

/// 根据扩展名分类文件类型。
pub fn classify_file_type(ext: &str) -> Option<String> {
    if ext.is_empty() {
        return None;
    }

    let ext_lower = ext.to_lowercase();
    let category = match ext_lower.as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
        | "3gp" | "rmvb" | "ts" => "VIDEO",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        | "heic" | "raw" | "cr2" | "nef" | "psd" => "IMAGE",
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => "AUDIO",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "csv"
        | "json" | "xml" | "html" | "htm" | "rtf" | "odt" | "ods" | "odp" | "epub" | "mobi"
        | "log" | "yaml" | "yml" | "toml" | "ini" | "cfg" => "DOCUMENT",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "lz" | "lz4" | "zst" | "iso"
        | "cab" | "arj" | "apk" | "dmg" => "ARCHIVE",
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

/// 将路径的元数据填充到 FileNode 中。
/// 接受可选的 `&fs::Metadata` 以复用 WalkDir 已缓存的 metadata。
fn build_file_node(path: &Path, parent_id: Option<String>, meta: Option<&fs::Metadata>) -> FileNode {
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
        (0, ext, ftype)
    };

    let (actual_size, created_at, modified_at, accessed_at) = match meta {
        Some(m) => {
            let size_val = if is_directory { 0 } else { m.len() };
            let created = m
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let modified = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let accessed = m
                .accessed()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (size_val, created, modified, accessed)
        }
        None => (0, 0, 0, 0),
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
        assert_eq!(classify_file_type("psd"), Some("IMAGE".into()));
    }

    #[test]
    fn test_classify_audio() {
        assert_eq!(classify_file_type("mp3"), Some("AUDIO".into()));
        assert_eq!(classify_file_type("wav"), Some("AUDIO".into()));
        assert_eq!(classify_file_type("flac"), Some("AUDIO".into()));
    }

    #[test]
    fn test_classify_document() {
        assert_eq!(classify_file_type("pdf"), Some("DOCUMENT".into()));
        assert_eq!(classify_file_type("xlsx"), Some("DOCUMENT".into()));
    }

    #[test]
    fn test_classify_archive() {
        assert_eq!(classify_file_type("zip"), Some("ARCHIVE".into()));
        assert_eq!(classify_file_type("apk"), Some("ARCHIVE".into()));
        assert_eq!(classify_file_type("dmg"), Some("ARCHIVE".into()));
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
    fn test_generate_id_is_uuid_v5() {
        let id = generate_id("C:\\Users\\test\\file.txt");
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|c| *c == '-').count(), 4);
        // 确定性
        let id2 = generate_id("C:\\Users\\test\\file.txt");
        assert_eq!(id, id2);
        // 唯一性
        let id3 = generate_id("C:\\Users\\test\\other.txt");
        assert_ne!(id, id3);
    }
}
