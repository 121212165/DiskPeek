# DiskPeek 重构计划书

> 基于 5 个角色代理的并行分析综合产出：
> - 🏗️ 系统架构师
> - 🦀 Rust 后端专家
> - 🎨 Svelte 前端专家
> - 🔒 安全与性能审查员
> - 📱 产品/UX 专家

---

## 一、当前项目概要

| 维度 | 现状 |
|------|------|
| 技术栈 | Tauri 2 (Rust) + Svelte 4 + TypeScript + D3.js v7 |
| 后端 | 3 个 Rust 文件 (650 行)，4 个 Tauri 命令 |
| 前端 | 6 个 Svelte 组件 + 3 个 TS 模块 (1,425 行) |
| 状态管理 | 全部集中在 App.svelte，通过 props 逐层传递 |
| 测试 | Rust: 8 个简单单元测试；前端: 0 |
| CI/CD | GitHub Actions (build + lint + audit + release) |

---

## 二、核心问题汇总（按严重度排序）

### 🔴 P0 — 立即修复（功能损坏）

| # | 问题 | 影响 | 涉及文件 | 修复方案 |
|---|------|------|---------|---------|
| 1 | **Svelte 响应式断裂** — `rootNodes.push(node)` 不触发 UI 更新 | 扫描期间列表和状态栏完全不更新，用户看不到进度 | `App.svelte:68-70` | 改为 `rootNodes = [...rootNodes, ...payload.nodes]` |
| 2 | **事件节点扁平推入** — 深层目录节点也被推入 rootNodes 顶层 | 树形数据结构被破坏，节点重复 | `App.svelte:65-72` + `scanner.rs` | 只处理顶层节点，或递归合并到现有树 |

### 🟠 P1 — 高优先级（性能/体验严重缺陷）

| # | 问题 | 影响 | 涉及文件 | 修复方案 |
|---|------|------|---------|---------|
| 3 | **双重文件系统遍历** — walkdir 遍历后 collect_children 又 fs::read_dir 一次 | 扫描时间翻倍，500K 文件 = 50 万次多余系统调用 | `scanner.rs:104-186, 318-344` | 栈驱动单次遍历算法（Node.js 端或 Rust 端） |
| 4 | **metadata 未复用** — walkdir 已有 metadata 但另调 `fs::metadata` | 每次额外 stat 系统调用 | `scanner.rs:273` | 改用 `entry.metadata()` |
| 5 | **SHA256 ID 碰撞** — 取前 8 字节 (64bit)，77K 文件后预期碰撞 | 前端 `key` 重复，DOM 元素重复/丢失 | `scanner.rs:244-249` | 使用 UUID v5 或完整 SHA256 |
| 6 | **无取消机制** — 扫描一旦启动无法中断 | 用户失去控制权，大型扫描只能强制关闭 | `main.rs:69` | `Arc<AtomicBool>` 取消标志 |
| 7 | **全量 JSON 序列化** — 每个事件携带完整子树 | 扫描时间 15-30% 浪费在序列化 | `main.rs:96-107` | 增量发送，分批控制 ≤50 文件/批 |

### 🟡 P2 — 中优先级（架构/质量改进）

| # | 问题 | 影响 | 涉及文件 | 修复方案 |
|---|------|------|---------|---------|
| 8 | **状态管理集中** — 6 个状态变量在 App.svelte，prop drilling | 新增状态需修改整个组件链 | `App.svelte:13-18` | 提取 Svelte store: `stores/scan.ts`, `stores/ui.ts` |
| 9 | **Treemap/Sunburst 代码重复** — breadcrumb + tooltip 逻辑 50% 重复 | 649 行中有 ~300 行重复 | `TreemapView.svelte`, `SunburstView.svelte` | 提取 `D3DrillDown.svelte` 共享容器 |
| 10 | **同步大文件缓存写入** — save_cache 阻塞扫描线程 | 扫描尾部延迟，mpsc buffer 满时子线程阻塞 | `main.rs:122` | 独立线程异步写入 + `to_string` 替代 `to_string_pretty` |
| 11 | **ListView 全量重计算** — O(N²) 累计复杂度 | 扫描期间列表卡顿 | `ListView.svelte:42-78` | 增量插入 + 二分查找 + 防抖 |
| 12 | **D3 视图每次完全重建 DOM** — `remove()` 后重建 | 视图切换和钻取卡顿 | `TreemapView.svelte:119`, `SunburstView.svelte:115` | 移除 `remove()`，使用标准 enter/update/exit |
| 13 | **无错误处理类型** — 全程 `eprintln!` 手动日志 | 错误无法传播、区分、恢复 | 整个 Rust 后端 | 添加 `thiserror` + `anyhow` |

### 🔵 P3 — 产品缺陷（功能缺口）

| # | 问题 | 影响 | 建议方案 |
|---|------|------|---------|
| 14 | **Treemap/Sunburst 未启用** — 代码已完工作处于占位符状态 | 两个核心视图不可用 | `App.svelte` 中导入组件替换 `placeholder` |
| 15 | **无搜索/过滤** | 用户无法在千级文件中快速定位 | 纯前端 `searchQuery` store + 实时过滤 |
| 16 | **无进度百分比** — 只有"已发现 N 个文件" | 用户无法预估剩余时间 | Rust 端报告扫描进度百分比 |
| 17 | **无暗色模式** | 桌面工具用户期望 | CSS 自定义属性 + `prefers-color-scheme` |
| 18 | **UI 硬编码中文** | 无法国际化 | 提取 `i18n.ts` key-value 映射 |

### 🟢 P4 — 防御性改进（安全/质量）

| # | 问题 | 影响 | 修复方案 |
|---|------|------|---------|
| 19 | **CSP 被完全禁用** (`"csp": null`) | 无 XSS 防御纵深 | 配置合理的 CSP |
| 20 | **explorer 路径未净化** | 不可见字符路径导致 explorer 异常 | `canonicalize()` + 空字节过滤 |
| 21 | **无前端测试** | 格式化工具函数无验证 | Vitest + 纯函数测试 |
| 22 | **Rust 测试覆盖率低** | 扫描引擎核心无集成测试 | tempfile + 临时目录集成测试 |
| 23 | **`formatSize` 重复** — colors.ts 和 format.ts 各有一份 | 代码分歧 | colors.ts 移除，改 import |

---

## 三、统一重构计划（四阶段）

### Phase 1 — 紧急修复（1 天）🆘

> 目标：恢复核心功能正常运作，消除"看不到进度"和"数据结构损坏"两个致命 Bug

| 步骤 | 改动 | 涉及文件 |
|------|------|---------|
| 1.1 | `rootNodes.push` → `rootNodes = [...rootNodes, ...]` | `App.svelte:68-70` |
| 1.2 | 事件处理仅接受顶层节点或递归合并树结构 | `App.svelte`, `scanner.rs` |
| 1.3 | 启用 TreemapView 和 SunburstView (替换 placeholder) | `App.svelte:120-124` |
| 1.4 | 添加 `cancel_scan` Tauri 命令 + 取消按钮 | `main.rs`, `Toolbar.svelte` |
| 1.5 | SHA256 截断改 UUID v5 | `scanner.rs:244-249` + `Cargo.toml` |

### Phase 2 — 架构重构（3-4 天）🏗️

> 目标：重构前后端架构，消除核心性能瓶颈和代码重复

| 步骤 | 改动 | 涉及文件 |
|------|------|---------|
| 2.1 | 提取 Svelte store: `stores/scan.ts`, `stores/ui.ts` | 新建 2 个文件 + 修改所有组件 |
| 2.2 | 提取 `D3DrillDown.svelte` 共享容器 (breadcrumb + tooltip) | 新建 1 个文件 + 重构 2 个 D3 视图 |
| 2.3 | 扫描引擎重构：栈驱动单次遍历 + metadata 复用 | `scanner.rs` (重写核心算法) |
| 2.4 | 缓存写入异步化：独立线程 + 无缩进序列化 | `main.rs:114-122` |
| 2.5 | ListView 增量插入优化 (二分查找 + 防抖) | `ListView.svelte:42-78` |
| 2.6 | D3 视图移除 `.remove()` 改用 join 增量更新 | `TreemapView.svelte:119`, `SunburstView.svelte:115` |

### Phase 3 — 模块拆分与质量（2-3 天）🧹

> 目标：代码组织清晰化，错误处理统一化，测试覆盖

| 步骤 | 改动 | 涉及文件 |
|------|------|---------|
| 3.1 | main.rs 拆分为 commands/ + error.rs + events | `main.rs` → `commands/`, `error.rs` |
| 3.2 | thiserror + anyhow 统一错误处理 | 新建 `error.rs`，修改所有 Rust 文件 |
| 3.3 | CSP 配置合理化 | `tauri.conf.json:24` |
| 3.4 | `open_in_explorer` 路径安全检查 | `main.rs:175-181` |
| 3.5 | Rust 集成测试 (tempfile + 临时目录) | `tests/integration.rs` |
| 3.6 | 前端 Vitest 测试 (format.ts, colors.ts) | `vitest.config.ts` + `*.spec.ts` |

### Phase 4 — 产品功能（2-3 周）🚀

> 目标：从"能用的工具"升级为"令人愉快的桌面应用"

| 步骤 | 改动 | 优先级 |
|------|------|--------|
| 4.1 | 搜索/过滤 (searchQuery store + 实时过滤) | P0 |
| 4.2 | 文件类型筛选器 (按钮组: 全部/视频/图片/文档/...) | P1 |
| 4.3 | 进度百分比 (Rust 端报告进度) | P1 |
| 4.4 | 暗色/亮色模式 (CSS custom properties) | P2 |
| 4.5 | i18n 字符串表 (i18n.ts) | P2 |
| 4.6 | 仪表盘概览页 (环形图 + Top 10 + 类型分布) | P3 |
| 4.7 | 键盘快捷键 (Ctrl+F, ←/→, Esc) | P3 |
| 4.8 | 右键菜单 (打开位置/复制路径) | P3 |

---

## 四、技术债务总结

### 依赖变化

| 操作 | Crate | 原因 |
|------|-------|------|
| 移除 | `sha2`, `hex` | 被 `uuid` 替代 |
| 新增 | `uuid` (v5) | SHA256 截断碰撞风险 |
| 新增 | `thiserror` | 统一错误类型 |
| 新增 | `anyhow` | 应用层错误传播 |
| 新增(dev) | `tempfile` | 集成测试创建临时目录 |

### 预期代码量变化

| 阶段 | 新增 | 删除/重构 | 净变化 |
|------|------|-----------|--------|
| Phase 1 | ~50 行 | ~20 行 | +30 行 |
| Phase 2 | ~300 行 | ~200 行 | +100 行 |
| Phase 3 | ~250 行 | ~150 行 | +100 行 |
| Phase 4 | ~500 行 | ~50 行 | +450 行 |
| **总计** | **~1,100 行** | **~420 行** | **+680 行** |

---

## 五、最终建议

1. **从 Phase 1 开始** — P0 问题 (Svelte 响应式断裂 + 节点扁平化) 是"功能不能用"的级别，必须在其他任何工作之前修复
2. **Phase 1 + 2 可并行** — 错误代理 (前端) 修复 Svelte Bug，Rust 代理重构扫描引擎，互不阻塞
3. **尽快启用 Treemap/Sunburst** — 代码已存在，接线只需几分钟，立即获得两个核心视图
4. **不要过早 i18n** — 中文单语言版本先打磨核心体验，国际化在 Phase 4

---

*生成于 2026-05-27，基于 5 角色代理并行分析*
