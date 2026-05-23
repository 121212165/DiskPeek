// --- FileNode ---

export interface FileNode {
  id: string;
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  extension: string;
  fileType: 'VIDEO' | 'IMAGE' | 'DOCUMENT' | 'ARCHIVE' | 'OTHER' | null;
  createdAt: number;
  modifiedAt: number;
  accessedAt: number;
  children: FileNode[];
  parentId: string | null;
  permissionDenied: boolean;
}

// --- Scan Event (from Rust backend) ---

export interface ScanEvent {
  rootPath: string;
  nodes: FileNode[];
}

// --- Scan Cache (persisted) ---

export interface ScanCache {
  scanTime: number;
  scanMode: string;
  rootNodes: FileNode[];
  totalFileCount: number;
  totalSize: number;
}

// --- Enums / Union Types ---

export type ViewMode = 'list' | 'treemap' | 'sunburst';

export type ScanState = 'idle' | 'scanning' | 'done';