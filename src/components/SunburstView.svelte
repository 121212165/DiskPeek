<script lang="ts">
  import * as d3 from 'd3';
  import type { FileNode } from '../lib/types';
  import { getNodeColor, formatSize, TYPE_COLORS } from "../lib/colors";

  // --- Props ---
  export let rootNodes: FileNode[] = [];
  export let onOpenFile: (path: string) => void = () => {};

  const MAX_DEPTH = 5;

  // Cap depth to MAX_DEPTH
  function capDepth(node: FileNode, depth: number): FileNode {
    if (!node.isDirectory || !node.children || node.children.length === 0) return node;
    if (depth >= MAX_DEPTH - 1) {
      const total = node.children.reduce((s, c) => s + c.size, 0);
      return {
        ...node,
        children: [{
          id: node.id + '__merged',
          name: '\u5176\u4ed6',
          path: node.path + '/__merged__',
          isDirectory: false,
          size: total,
          extension: '',
          fileType: 'OTHER',
          createdAt: 0, modifiedAt: 0, accessedAt: 0,
          children: [],
          parentId: node.id,
          permissionDenied: false,
        }],
      };
    }
    return { ...node, children: node.children.map(c => capDepth(c, depth + 1)) };
  }

  // --- State ---
  let currentRoot: FileNode | null = null;
  let breadcrumb: FileNode[] = [];
  let svgEl: SVGSVGElement | null = null;

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
    if (rootNodes.length === 1) return capDepth(rootNodes[0], 0);
    return capDepth({
      id: '__virtual__',
      name: '\u5168\u90e8',
      path: '',
      isDirectory: true,
      size: rootNodes.reduce((s, n) => s + n.size, 0),
      extension: '', fileType: null,
      createdAt: 0, modifiedAt: 0, accessedAt: 0,
      children: rootNodes,
      parentId: null, permissionDenied: false,
    }, 0);
  })();

  $: dataRoot = currentRoot ?? wrappedRoot;

  // --- Render ---
  $: if (dataRoot && svgEl) {
    const root = dataRoot;
    const el = svgEl;
    const width = el.clientWidth;
    const height = el.clientHeight;
    if (width <= 0 || height <= 0) return;

    const hierarchy = d3.hierarchy<FileNode>(root)
      .sum(d => d.size)
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    const totalValue = hierarchy.value ?? 1;
    const depth = hierarchy.height;
    const radius = Math.min(width, height) / 2;

    const partition = d3.partition<FileNode>()
      .size([2 * Math.PI, radius]);

    partition(hierarchy);

    const arcGen = d3.arc<d3.HierarchyRectangularNode<FileNode>>()
      .startAngle(d => d.x0)
      .endAngle(d => d.x1)
      .innerRadius(d => (d.depth / (depth + 1)) * radius)
      .outerRadius(d => ((d.depth + 1) / (depth + 1)) * radius);

    const svg = d3.select(el);
    svg.selectAll('g').remove();
    const g = svg.append('g')
      .attr('transform', `translate(${width / 2},${height / 2})`);

    const arcs = hierarchy.descendants().filter(d => d.depth > 0);

    // Enter + update via join
    const path = g.selectAll<SVGPathElement, d3.HierarchyRectangularNode<FileNode>>('path')
      .data(arcs, d => (d.data as FileNode).id)
      .join(
        enter => {
          const ep = enter.append('path')
            .attr('fill', d => getNodeColor((d.data as FileNode).name, (d.data as FileNode).isDirectory))
            .attr('stroke', '#fff')
            .attr('stroke-width', 1)
            .attr('opacity', 0)
            .attr('d', d => arcGen(d) as string);

          ep.transition().duration(500).attr('opacity', 0.9);
          return ep;
        },
        update => update,
        exit => exit.transition().duration(500).attr('opacity', 0).remove()
      );

    // Animate arc positions
    path.transition().duration(500)
      .attr('d', d => arcGen(d) as string);

    // Mouse events
    path.on('mouseenter', function (event: MouseEvent, d) {
      const node = d.data as FileNode;
      showTooltip(event, node, d.value ?? 0, totalValue);
    });
    path.on('mousemove', function (event: MouseEvent) {
      moveTooltip(event);
    });
    path.on('mouseleave', hideTooltip);

    // Click to drill down
    path.on('click', function (_event, d) {
      const node = d.data as FileNode;
      if (node.isDirectory && node.children && node.children.length > 0) {
        breadcrumb = [...breadcrumb, node];
        currentRoot = node;
      } else if (!node.isDirectory) {
        onOpenFile(node.path);
      }
    });

    // Remove old labels
    g.selectAll('text').remove();

    // Add new labels
    path.each(function (d) {
      const arcLen = d.x1 - d.x0;
      if (arcLen < 0.2) return;

      const midAngle = (d.x0 + d.x1) / 2;
      const labelRadius = ((d.depth + 0.65) / (depth + 1)) * radius;
      const x = Math.sin(midAngle) * labelRadius;
      const y = -Math.cos(midAngle) * labelRadius;
      const nodeName = (d.data as FileNode).name;

      const textEl = g.append('text')
        .attr('x', x)
        .attr('y', y)
        .attr('fill', '#fff')
        .attr('font-size', Math.min(13, Math.max(9, (arcLen * radius) / 10)))
        .attr('text-anchor', 'middle')
        .attr('dominant-baseline', 'middle')
        .text(nodeName.length > 12 ? nodeName.slice(0, 10) + '..' : nodeName);

      // Rotate text along arc
      const angleDeg = midAngle * 180 / Math.PI - 90;
      textEl.attr('transform', `rotate(${angleDeg},${x},${y})`);
    });

    // Center label
    g.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('fill', '#374151')
      .attr('font-size', 14)
      .attr('font-weight', 600)
      .text((hierarchy.data as FileNode).name.length > 10
        ? (hierarchy.data as FileNode).name.slice(0, 8) + '..'
        : (hierarchy.data as FileNode).name);
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
  <div class="sunburst-container">
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

    <div class="svg-wrapper">
      <svg bind:this={svgEl} viewBox="-50 -50 100 100" preserveAspectRatio="xMidYMid meet"></svg>
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
  .sunburst-container {
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