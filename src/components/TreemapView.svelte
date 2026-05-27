<script lang="ts">
  import * as d3 from 'd3';
  import type { FileNode } from '../lib/types';
  import { getNodeColor } from '../lib/colors';
  import { formatSize } from '../lib/format';
  import D3DrillDown from '../lib/d3/D3DrillDown.svelte';

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

  let svgEl: SVGSVGElement | null = null;
  let svgWidth = 800;
  let svgHeight = 600;

  // Render function called by D3DrillDown slot
  function renderChart(dataRoot: FileNode | null, showTooltip: Function, moveTooltip: Function, hideTooltip: Function, drillDown: Function) {
    if (!dataRoot || !svgEl) return;
    const el = svgEl;
    const width = el.clientWidth;
    const height = el.clientHeight;
    if (width <= 0 || height <= 0) return;

    const capped = capNodes(dataRoot, { n: 0 });
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

    cell.select('rect')
      .transition().duration(500)
      .attr('x', d => d.x0)
      .attr('y', d => d.y0)
      .attr('width', d => Math.max(0, d.x1 - d.x0))
      .attr('height', d => Math.max(0, d.y1 - d.y0));

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

    cell.on('mouseenter', function (event: MouseEvent, d) {
      const node = d.data as FileNode;
      showTooltip(event, node.name, node.size, ((d.value ?? 0) / totalValue * 100).toFixed(1));
    });
    cell.on('mousemove', function (event: MouseEvent) {
      moveTooltip(event);
    });
    cell.on('mouseleave', hideTooltip);

    cell.on('click', function (_event, d) {
      const node = d.data as FileNode;
      drillDown(node);
    });
  }
</script>

<D3DrillDown {rootNodes} {onOpenFile} let:dataRoot let:drillDown let:showTooltip let:moveTooltip let:hideTooltip>
  <div class="svg-wrapper" bind:clientWidth={svgWidth} bind:clientHeight={svgHeight}>
    <svg
      bind:this={svgEl}
      viewBox="0 0 {svgWidth} {svgHeight}"
      preserveAspectRatio="xMidYMid meet"
    ></svg>
  </div>
  <!-- Trigger render when dataRoot changes -->
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
