<script lang="ts">
  import type { FileNode } from '../types';
  import { formatSize } from '../format';

  export let rootNodes: FileNode[];
  export let rootName: string = '全部';
  export let onOpenFile: (path: string) => void = () => {};

  // Breadcrumb state
  let currentRoot: FileNode | null = null;
  let breadcrumb: FileNode[] = [];

  // Tooltip state
  let tooltipVisible = false;
  let tooltipX = 0;
  let tooltipY = 0;
  let tooltipName = '';
  let tooltipSize = '';
  let tooltipPercent = '';

  export function showTooltip(event: MouseEvent, name: string, sizeBytes: number, percent: string) {
    tooltipX = event.clientX;
    tooltipY = event.clientY;
    tooltipName = name;
    tooltipSize = formatSize(sizeBytes);
    tooltipPercent = percent;
    tooltipVisible = true;
  }

  export function moveTooltip(event: MouseEvent) {
    tooltipX = event.clientX;
    tooltipY = event.clientY;
  }

  export function hideTooltip() {
    tooltipVisible = false;
  }

  export function drillDown(node: FileNode) {
    if (node.isDirectory && node.children && node.children.length > 0) {
      breadcrumb = [...breadcrumb, node];
      currentRoot = node;
    } else if (!node.isDirectory) {
      onOpenFile(node.path);
    }
  }

  function goToBreadcrumb(index: number) {
    if (index < 0) {
      currentRoot = null;
      breadcrumb = [];
    } else {
      currentRoot = breadcrumb[index];
      breadcrumb = breadcrumb.slice(0, index);
    }
  }

  // Virtual root when multiple top-level roots
  $: wrappedRoot = (() => {
    if (!rootNodes || rootNodes.length === 0) return null;
    if (rootNodes.length === 1) return rootNodes[0];
    return {
      id: '__virtual__',
      name: rootName,
      path: '',
      isDirectory: true,
      size: rootNodes.reduce((s, n) => s + n.size, 0),
      extension: '',
      fileType: null,
      createdAt: 0,
      modifiedAt: 0,
      accessedAt: 0,
      children: rootNodes,
      parentId: null,
      permissionDenied: false,
    } as FileNode;
  })();

  $: dataRoot = currentRoot ?? wrappedRoot;
</script>

{#if rootNodes.length === 0}
  <div class="empty-state">暂无扫描数据</div>
{:else}
  <div class="chart-container">
    <div class="breadcrumb">
      <button
        class="crumb"
        class:active={currentRoot === null}
        on:click={() => goToBreadcrumb(-1)}
      >
        {wrappedRoot?.name ?? rootName}
      </button>
      {#each breadcrumb as node, i}
        <span class="sep">&gt;</span>
        <button class="crumb" on:click={() => goToBreadcrumb(i)}>
          {node.name}
        </button>
      {/each}
    </div>

    <div class="svg-wrapper">
      <slot {dataRoot} {drillDown} {showTooltip} {moveTooltip} {hideTooltip} />
    </div>

    {#if tooltipVisible}
      <div class="tooltip" style="left:{tooltipX + 12}px;top:{tooltipY - 10}px;">
        <div class="tt-name">{tooltipName}</div>
        <div class="tt-info">{tooltipSize} ({tooltipPercent}%)</div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #9ca3af;
    font-size: 16px;
  }

  .chart-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    position: relative;
  }

  .breadcrumb {
    height: 36px;
    display: flex;
    align-items: center;
    background: var(--bg-toolbar-secondary, #f3f4f6);
    padding: 0 12px;
    gap: 4px;
    flex-shrink: 0;
    overflow-x: auto;
    white-space: nowrap;
  }

  .crumb {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary, #374151);
    padding: 4px 8px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .crumb:hover {
    background: var(--bg-hover, #e5e7eb);
  }

  .crumb.active {
    font-weight: 600;
    color: var(--text-primary, #1f2937);
  }

  .sep {
    color: #9ca3af;
    font-size: 12px;
    flex-shrink: 0;
  }

  .svg-wrapper {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .svg-wrapper :global(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }

  .tooltip {
    position: fixed;
    background: var(--bg-card, #fff);
    color: var(--text-primary, #1f2937);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 8px 12px;
    pointer-events: none;
    z-index: 1000;
    font-size: 13px;
  }

  .tt-name {
    font-weight: 600;
    margin-bottom: 2px;
  }

  .tt-info {
    color: var(--text-secondary, #6b7280);
  }
</style>
