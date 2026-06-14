use serde::Serialize;
use std::path::Path;
use tiny_http::{Header, Method, Response, Server};
use walkdir::WalkDir;

const VIEWER_HTML: &str = include_str!("viewer.html");

#[derive(Debug, Clone, Serialize)]
struct FileNode {
    name: String,
    path: String,
    #[serde(rename = "isDir")]
    is_dir: bool,
    size: u64,
    #[serde(rename = "fileType")]
    file_type: &'static str,
    #[serde(rename = "modifiedAt")]
    modified_at: u64,
    children: Vec<FileNode>,
}

fn main() {
    let port = 3030u16;
    let server = Server::http(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|e| panic!("Failed to start server on port {}: {}", port, e));
    println!("DiskPeek running at http://localhost:{}", port);
    println!("Press Ctrl+C to stop.");
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let response = match (request.method(), url.as_str()) {
            (Method::Get, "/") => html_response(VIEWER_HTML),
            (Method::Get, u) if u.starts_with("/api/scan") => scan_response(&url),
            _ => not_found(),
        };
        let _ = request.respond(response);
    }
}

fn html_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_header(
            Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap(),
        )
        .with_header(
            Header::from_bytes(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline'",
            )
            .unwrap(),
        )
}

fn json_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(
        Header::from_bytes("Content-Type", "application/json").unwrap(),
    )
}

fn not_found() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("Not Found").with_status_code(404)
}

fn scan_response(url: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let scan_path = url
        .split('?')
        .nth(1)
        .and_then(|q| q.strip_prefix("path="))
        .map(|p| p.split('&').next().unwrap_or(p))
        .map(urlencoding::decode)
        .and_then(|r| r.ok())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| Path::new(".").to_path_buf())
                .to_string_lossy()
                .to_string()
        });
    let root = scan_dir(&scan_path, 3);
    let node_count = count_nodes(&root);
    let total_size = root.iter().map(|n| n.size).sum::<u64>();
    let response_body = serde_json::json!({
        "nodes": root,
        "nodeCount": node_count,
        "totalSize": total_size,
    });
    json_response(serde_json::to_string(&response_body).unwrap())
}

fn scan_dir(root: &str, max_depth: usize) -> Vec<FileNode> {
    let mut stack: Vec<(std::path::PathBuf, FileNode, Vec<FileNode>, usize)> = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            !matches!(
                name.as_str(),
                "node_modules"
                    | ".git"
                    | "__pycache__"
                    | "target"
                    | ".cache"
                    | ".npm"
                    | ".yarn"
                    | "vendor"
                    | "dist"
                    | "build"
            )
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let depth = entry.depth();
        if entry.file_type().is_symlink() {
            continue;
        }
        while stack.last().map_or(false, |(_, _, _, d)| *d >= depth) {
            let (_, mut dir_node, children, _) = stack.pop().unwrap();
            dir_node.children = children;
            if let Some(top) = stack.last_mut() {
                top.1.size += dir_node.size;
                top.2.push(dir_node);
            }
        }
        if stack.len() >= max_depth && entry.file_type().is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy().to_string();
        let is_dir = entry.file_type().is_dir();
        let meta = entry.metadata().ok();
        let (size, file_type) = if is_dir {
            (0, "")
        } else {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let size_val = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            (size_val, classify(&ext))
        };
        let modified_at = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let node = FileNode {
            name,
            path: path_str,
            is_dir,
            size,
            file_type,
            modified_at,
            children: Vec::new(),
        };
        if is_dir {
            stack.push((path.to_path_buf(), node, Vec::new(), depth));
        } else if let Some(top) = stack.last_mut() {
            top.1.size += size;
            top.2.push(node);
        }
    }
    while let Some((_, mut dir_node, children, _)) = stack.pop() {
        dir_node.children = children;
        if let Some(top) = stack.last_mut() {
            top.1.size += dir_node.size;
            top.2.push(dir_node);
        } else {
            return vec![dir_node];
        }
    }
    Vec::new()
}

fn count_nodes(nodes: &[FileNode]) -> usize {
    nodes
        .iter()
        .map(|n| 1 + count_nodes(&n.children))
        .sum::<usize>()
}

fn classify(ext: &str) -> &'static str {
    match ext {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_known_types() {
        assert_eq!(classify("mp4"), "VIDEO");
        assert_eq!(classify("png"), "IMAGE");
        assert_eq!(classify("mp3"), "AUDIO");
        assert_eq!(classify("pdf"), "DOCUMENT");
        assert_eq!(classify("zip"), "ARCHIVE");
        assert_eq!(classify("exe"), "OTHER");
        assert_eq!(classify(""), "OTHER");
    }
}
