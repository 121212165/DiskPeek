# DiskPeek

C盘扫描与可视化桌面应用，使用 Tauri (Rust) + Svelte + D3.js 构建。

## 功能特性

- **C盘扫描**：快速扫描C盘文件系统，生成文件树
- **多视图可视化**：
  - 列表视图：按文件大小、类型、修改时间排序
  - Treemap视图：矩形树图展示文件大小分布
  - 旭日图视图：层级结构可视化
- **缓存机制**：扫描结果缓存，提升重复扫描速度
- **资源管理器集成**：右键菜单快速打开文件位置

## 技术栈

- **前端**：Svelte 4 + TypeScript + Vite + D3.js
- **后端**：Rust + Tauri 2
- **构建工具**：Vite, Cargo

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建前端
npm run build

# Tauri开发
npm run tauri dev

# Tauri构建
npm run tauri build
```

## CI/CD

项目使用 GitHub Actions 进行持续集成：
- 前端构建检查
- Rust 代码检查与测试
- 安全依赖审计

## 许可证

MIT