<script lang="ts">
  import * as d3 from 'd3';
  import type { FileNode } from '../lib/types';
  import { getNodeColor } from '../lib/colors';
  import D3DrillDown from '../lib/d3/D3DrillDown.svelte';

  export let rootNodes: FileNode[] = [];
  export let onOpenFile: (path: string) => void = () => {};

  const MAX_DEPTH = 5;

  function capDepth(node: FileNode, depth: number): FileNode {
    if (!node.isDirectory || !node.children || node.children.length === 0) return node;
    if (depth >= MAX_DEPTH - 1) {
      const total = node.children.reduce((s, c) => s + c.size, 0);
      return {
        ...node,
        children: [{
          id: node.id + '__merged',
          name: '其他',
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

  let svgEl: SVGSVGElement | null = null;

  function renderChart(dataRoot: FileNode | null, showTooltip: Function, moveTooltip: Function, hideTooltip: Function, drillDown: Function) {
    if (!dataRoot || !svgEl) return;
    const el = svgEl;
    const width = el.clientWidth;
    const height = el.clientHeight;
    if (width <= 0 || height <= 0) return;

    const capped = capDepth(dataRoot, 0);

    const hierarchy = d3.hierarchy<FileNode>(capped)
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

    path.transition().duration(500)
      .attr('d', d => arcGen(d) as string);

    path.on('mouseenter', function (event: MouseEvent, d) {
      const node = d.data as FileNode;
      showTooltip(event, node.name, node.size, ((d.value ?? 0) / totalValue * 100).toFixed(1));
    });
    path.on('mousemove', function (event: MouseEvent) {
      moveTooltip(event);
    });
    path.on('mouseleave', hideTooltip);

    path.on('click', function (_event, d) {
      const node = d.data as FileNode;
      drillDown(node);
    });

    // Remove old labels
    g.selectAll('text').remove();

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

      const angleDeg = midAngle * 180 / Math.PI - 90;
      textEl.attr('transform', `rotate(${angleDeg},${x},${y})`);
    });

    // Center label
    g.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('fill', 'var(--text-primary, #374151)')
      .attr('font-size', 14)
      .attr('font-weight', 600)
      .text((hierarchy.data as FileNode).name.length > 10
        ? (hierarchy.data as FileNode).name.slice(0, 8) + '..'
        : (hierarchy.data as FileNode).name);
  }
</script>

<D3DrillDown {rootNodes} {onOpenFile} let:dataRoot let:drillDown let:showTooltip let:moveTooltip let:hideTooltip>
  <div class="svg-wrapper">
    <svg bind:this={svgEl} viewBox="-50 -50 100 100" preserveAspectRatio="xMidYMid meet"></svg>
  </div>
  {#if dataRoot && svgEl}
    {@html (renderChart(dataRoot, showTooltip, moveTooltip, hideTooltip, drillDown), '')}
  {/if}
</D3DrillDown>

<style>
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
</style>
