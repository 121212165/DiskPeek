<script lang="ts">
  import * as d3 from 'd3';
  import type { FileNode } from '../lib/types';
  import { getNodeColor, formatSize, TYPE_COLORS } from "../lib/colors";

  // --- Props ---
  export let rootNodes: FileNode[] = [];
  export let onOpenFile: (path: string) => void = () => {};

  const MAX_NODES = 3000;

  function capNodes(node: FileNode, count: { n: number }): FileNode {
    if (!node.isDirectory || !node.children || node.children.length === 0) return node;
    const capped: FileNode[] = [];
    let othersTotal = 0;
    for (const child of node.children) {
      if (count.n >= MAX_NODES) {
        othersTotal += child.size;
      } else {
        count.n++;
        capped.push(capNodes(child, count));
      }
    }
    if (othersTotal > 0) {
      capped.push({
        id: node.id + '__other',
        name: '其他',
        path: node.path + '/__other__',
        isDirectory: false,
        size: othersTotal,
        extension: '',
        fileType: 'OTHER',
        createdAt: 0, modifiedAt: 0, accessedAt: 0,
        children: [],
        parentId: node.id,
        permissionDenied: false,
      });
    }
    return { ...node, children: capped };
  }

  // --- State ---
  let currentRoot: FileNode | null = null;
  let breadcrumb: FileNode[] = [];
  let svgEl: SVGSVGElement | null = null;
  let svgWidth = 800;
  let svgHeight = 600;

  let tooltipVisible = false;
  let tooltipX = 0;
  let tooltipY = 0;
  let tooltipName = '';
  let tooltipSize = '';
  let tooltipPercent = '';

  function showTooltip(event: MouseEvent, node: FileNode, value: number, totalValue: number) {
    tooltipX = event.clientX;
    tooltipY = event.clientY;
    tooltipName = node.name;
    tooltipSize = formatSize(node.size);
    tooltipPercent = (value / totalValue * 100).toFixed(1);
    tooltipVisible = true;
  }

  function moveTooltip(event: MouseEvent) {
    tooltipX = event.clientX;
    tooltipY = event.clientY;
  }

  function hideTooltip() {
    tooltipVisible = false;
  }

  // --- Derived data ---
  $: wrappedRoot = (() => {
    if (!rootNodes || rootNodes.length === 0) return null;
    if (rootNodes.length === 1) return rootNodes[0];
    return {
      id: '__virtual__',
      name: '全部',
      path: '',
      isDirectory: true,
      size: rootNodes.reduce((s, n) => s + n.size, 0),
      extension: '', fileType: null,
      createdAt: 0, modifiedAt: 0, accessedAt: 0,
      children: rootNodes,
      parentId: null, permissionDenied: false,
    } as FileNode;
  })();

  $: dataRoot = currentRoot ?? wrappedRoot;

  // --- Render ---
  $: if (dataRoot && svgEl) {
    const root = dataRoot;
    const el = svgEl;
    const width = el.clientWidth;
    const height = el.clientHeight;
    if (width <= 0 || height <= 0) return;

    const capped = capNodes(root, { n: 0 });
    const hierarchy = d3.hierarchy<FileNode>(capped)
      .sum(d => d.size)
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    const totalValue = hierarchy.value ?? 1;

    const treemap = d3.treemap<FileNode>()
      .size([width, height])
      .paddingOuter(2)
      .paddingInner(2)
      .round(true);

    treemap(hierarchy);

    const leaves = hierarchy.leaves();
    const svg = d3.select(el);

    svg.selectAll('g').remove();
    const g = svg.append('g');

    // Enter + update via join
    const cell = g.selectAll<SVGGElement, d3.HierarchyRectangularNode<FileNode>>('g.cell')
      .data(leaves, d => (d.data as FileNode).id)
      .join(
        enter => {
          const eg = enter.append('g').attr('class', 'cell').attr('opacity', 0);

          eg.append('rect')
            .attr('fill', d => getNodeColor((d.data as FileNode).name, (d.data as FileNode).isDirectory))
            .attr('stroke', '#fff')
            .attr('stroke-width', 1)
            .attr('x', d => d.x0)
            .attr('y', d => d.y0)
            .attr('width', d => Math.max(0, d.x1 - d.x0))
            .attr('height', d => Math.max(0, d.y1 - d.y0));

          eg.append('text')
            .attr('fill', '#fff')
            .attr('font-size', d => {
              const area = (d.x1 - d.x0) * (d.y1 - d.y0);
              return Math.min(13, Math.max(8, Math.sqrt(area) / 6));
            })
            .attr('x', d => d.x0 + 4)
            .attr('y', d => d.y0 + 14)
            .text(d => {
              const area = (d.x1 - d.x0) * (d.y1 - d.y0);
              if (area < 200) return '';
              const name = (d.data as FileNode).name;
              return name.length > 18 ? name.slice(0, 16) + '..' : name;
            });

          eg.transition().duration(500).attr('opacity', 1);
          return eg;
        },
        update => update,
        exit => exit.transition().duration(500).attr('opacity', 0).remove()
      );

    // Animate rect positions
    cell.select('rect')
      .transition().duration(500)
      .attr('x', d => d.x0)
      .attr('y', d => d.y0)
      .attr('width', d => Math.max(0, d.x1 - d.x0))
      .attr('height', d => Math.max(0, d.y1 - d.y0));

    // Animate text positions
    cell.select('text')
      .transition().duration(500)
      .attr('x', d => d.x0 + 4)
      .attr('y', d => d.y0 + 14)
      .attr('font-size', d => {
        const area = (d.x1 - d.x0) * (d.y1 - d.y0);
        return Math.min(13, Math.max(8, Math.sqrt(area) / 6));
      })
      .tween('text', function (d) {
        const area = (d.x1 - d.x0) * (d.y1 - d.y0);
        const el = d3.select(this);
        return function () {
          if (area < 200) el.text('');
          else {
            const name = (d.data as FileNode).name;
            el.text(name.length > 18 ? name.slice(0, 16) + '..' : name);
          }
        };
      });

    // Mouse events
    cell.on('mouseenter', function (event: MouseEvent, d) {
      const node = d.data as FileNode;
      showTooltip(event, node, d.value ?? 0, totalValue);
    });
    cell.on('mousemove', function (event: MouseEvent) {
      moveTooltip(event);
    });
    cell.on('mouseleave', hideTooltip);

    // Click to drill down
    cell.on('click', function (_event, d) {
      const node = d.data as FileNode;
      if (node.isDirectory && node.children && node.children.length > 0) {
        breadcrumb = [...breadcrumb, node];
        currentRoot = node;
      } else if (!node.isDirectory) {
        onOpenFile(node.path);
      }
    });
  }

  // --- Navigation ---
  function goToBreadcrumb(index: number) {
    if (index < 0) {
      currentRoot = null;
      breadcrumb = [];
    } else {
      currentRoot = breadcrumb[index];
      breadcrumb = breadcrumb.slice(0, index);
    }
  }
</script>

{#if rootNodes.length === 0}
  <div class="empty-state">暂无扫描数据</div>
{:else}
  <div class="treemap-container">
    <div class="breadcrumb">
      <button class="crumb" class:active={currentRoot === null} on:click={() => goToBreadcrumb(-1)}>
        {wrappedRoot?.name ?? '全部'}
      </button>
      {#each breadcrumb as node, i}
        <span class="sep">&gt;</span>
        <button class="crumb" on:click={() => goToBreadcrumb(i)}>
          {node.name}
        </button>
      {/each}
    </div>

    <div class="svg-wrapper" bind:clientWidth={svgWidth} bind:clientHeight={svgHeight}>
      <svg bind:this={svgEl} viewBox="0 0 {svgWidth} {svgHeight}" preserveAspectRatio="xMidYMid meet"></svg>
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
  .treemap-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    position: relative;
  }
  .breadcrumb {
    height: 36px;
    display: flex;
    align-items: center;
    background: #f3f4f6;
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
    color: #374151;
    padding: 4px 8px;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .crumb:hover {
    background: #e5e7eb;
  }
  .crumb.active {
    font-weight: 600;
    color: #1f2937;
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
    background: #fff;
    color: #1f2937;
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
    color: #6b7280;
  }
</style>