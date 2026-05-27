<script lang="ts">
  import type { FileNode } from '../lib/types';
  import { formatSize, formatTime, getFileTypeIcon, formatPercent } from '../lib/format';

  export let rootNodes: FileNode[];
  export let scanning: boolean;
  export let onOpenFile: (path: string) => void;

  // --- Sort state ---

  type SortKey = 'name' | 'size';
  type SortDir = 'asc' | 'desc';

  let sortKey: SortKey = 'name';
  let sortDirection: SortDir = 'asc';

  // --- Expanded state ---

  let expandedIds: Set<string> = new Set();

  // Auto-expand first level
  $: {
    if (rootNodes.length > 0 && expandedIds.size === 0) {
      const ids = new Set<string>();
      for (const node of rootNodes) {
        if (node.permissionDenied || !node.isDirectory) continue;
        ids.add(node.id);
      }
      if (ids.size > 0) {
        expandedIds = ids;
      }
    }
  }

  // --- Flatten, sort, and compute grand total in one pass ---

  interface FlatRow {
    node: FileNode;
    depth: number;
  }

  function sumDirectorySize(node: FileNode): number {
    if (!node.isDirectory) return node.size;
    return node.children.reduce((sum, child) => sum + sumDirectorySize(child), 0);
  }

  function flattenAndSort(
    nodes: FileNode[],
    depth: number
  ): { rows: FlatRow[]; total: number } {
    const result: FlatRow[] = [];
    let total = 0;
    const stack: { items: FileNode[]; depth: number }[] = [{ items: nodes, depth }];

    while (stack.length > 0) {
      const { items, depth: d } = stack.pop()!;
      const sorted = [...items].sort((a, b) => {
        if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
        let cmp = sortKey === 'name' ? a.name.localeCompare(b.name) : a.size - b.size;
        return sortDirection === 'asc' ? cmp : -cmp;
      });
      for (let i = sorted.length - 1; i >= 0; i--) {
        const node = sorted[i];
        result.push({ node, depth: d });
        total += node.size;
        if (node.isDirectory && node.children.length > 0 && expandedIds.has(node.id)) {
          stack.push({ items: node.children, depth: d + 1 });
        }
      }
    }
    return { rows: result, total };
  }

  let grandTotal = 0;
  let flatRows: FlatRow[] = [];

  $: {
    const { rows, total } = flattenAndSort(rootNodes, 0);
    flatRows = rows;
    grandTotal = total;
  }

  // --- Handlers ---

  function toggleSort(key: SortKey): void {
    if (sortKey === key) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = key;
      sortDirection = 'asc';
    }
  }

  function sortIndicator(key: SortKey): string {
    if (sortKey !== key) return '';
    return sortDirection === 'asc' ? ' ▲' : ' ▼';
  }

  function toggleExpand(id: string): void {
    const next = new Set(expandedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedIds = next;
  }

  function handleRowClick(node: FileNode): void {
    if (!node.isDirectory && !node.permissionDenied) {
      onOpenFile(node.path);
    }
  }
</script>

<div class="list-container">
  {#if rootNodes.length === 0 && !scanning}
    <div class="empty-state">暂无扫描数据</div>
  {:else}
    <!-- Column headers -->
    <div class="header-row">
      <span class="col-name" on:click={() => toggleSort('name')} role="button" tabindex="0">
        名称{sortIndicator('name')}
      </span>
      <span class="col-size" on:click={() => toggleSort('size')} role="button" tabindex="0">
        大小{sortIndicator('size')}
      </span>
      <span class="col-time">修改时间</span>
      <span class="col-pct">占比</span>
    </div>

    <!-- Rows -->
    <div class="rows">
      {#each flatRows as row (row.node.id)}
        <div
          class="row"
          class:dir={row.node.isDirectory}
          class:denied={row.node.permissionDenied}
          style="padding-left: {row.depth * 20 + 8}px"
          on:click={() => handleRowClick(row.node)}
          role="button"
          tabindex="0"
        >
          <!-- Expand arrow -->
          <span class="arrow-cell">
            {#if row.node.isDirectory && row.node.children.length > 0 && !row.node.permissionDenied}
              <button
                class="arrow-btn"
                class:expanded={expandedIds.has(row.node.id)}
                on:click={(e) => { e.stopPropagation(); toggleExpand(row.node.id); }}
              >
                ▶
              </button>
            {:else if row.node.permissionDenied}
              <span class="lock-icon">🔒</span>
            {/if}
          </span>

          <!-- Icon + Name -->
          <span class="name-cell">
            <span class="node-icon">{row.node.isDirectory ? '📁' : getFileTypeIcon(row.node.fileType)}</span>
            <span class="node-name" class:denied={row.node.permissionDenied}>{row.node.name}</span>
          </span>

          <!-- Size -->
          <span class="size-cell">{row.node.isDirectory ? formatSize(sumDirectorySize(row.node)) : formatSize(row.node.size)}</span>

          <!-- Time -->
          <span class="time-cell">{formatTime(row.node.modifiedAt)}</span>

          <!-- Percent -->
          <span class="pct-cell">
            {row.node.isDirectory ? '-' : formatPercent(row.node.size, grandTotal)}
          </span>
        </div>
      {/each}
    </div>

    {#if scanning}
      <div class="scan-loader">
        <span class="loader-dot"></span>
        <span>正在扫描文件系统...</span>
      </div>
    {/if}
  {/if}
</div>

<style>
  .list-container {
    background: var(--bg-card, #fff);
    margin: 12px 16px 40px 16px;
    border-radius: 12px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
    overflow: hidden;
  }

  .empty-state {
    padding: 80px 0;
    text-align: center;
    color: #aaa;
    font-size: 15px;
  }

  /* --- Header --- */

  .header-row {
    display: flex;
    align-items: center;
    height: 40px;
    padding: 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary, #666);
    background: var(--bg-toolbar-secondary, #fafafa);
    border-bottom: 1px solid var(--border, #eee);
  }

  .col-name {
    flex: 1;
    cursor: pointer;
    user-select: none;
    padding-left: 28px;
  }

  .col-size {
    width: 100px;
    text-align: right;
    cursor: pointer;
    user-select: none;
  }

  .col-time {
    width: 160px;
    text-align: right;
  }

  .col-pct {
    width: 80px;
    text-align: right;
  }

  /* --- Rows --- */

  .rows {
    max-height: calc(100vh - 240px);
    overflow-y: auto;
  }

  .row {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 12px;
    font-size: 14px;
    border-bottom: 1px solid #f3f3f3;
    cursor: default;
    transition: background 0.1s;
  }

  .row:hover {
    background: var(--bg-hover, #f5f3ff);
  }

  .row.dir {
    font-weight: 500;
  }

  .row.denied {
    opacity: 0.55;
  }

  /* Arrow */

  .arrow-cell {
    width: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .arrow-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 10px;
    cursor: pointer;
    padding: 2px;
    line-height: 1;
    transition: transform 0.2s;
  }

  .arrow-btn.expanded {
    transform: rotate(90deg);
  }

  .lock-icon {
    font-size: 12px;
  }

  /* Name */

  .name-cell {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .node-icon {
    flex-shrink: 0;
  }

  .node-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .node-name.denied {
    color: #aaa;
  }

  /* Size */

  .size-cell {
    width: 100px;
    text-align: right;
    color: #555;
    font-variant-numeric: tabular-nums;
  }

  /* Time */

  .time-cell {
    width: 160px;
    text-align: right;
    color: #888;
    font-size: 13px;
  }

  /* Percent */

  .pct-cell {
    width: 80px;
    text-align: right;
    color: #999;
    font-size: 13px;
  }

  /* --- Scan loader --- */

  .scan-loader {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    font-size: 13px;
    color: var(--accent, #7c3aed);
    background: var(--bg-toolbar-secondary, #fafafa);
  }

  .loader-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #7c3aed;
    animation: pulse 1s infinite ease-in-out;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; transform: scale(0.8); }
    50% { opacity: 1; transform: scale(1); }
  }
</style>
