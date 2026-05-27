<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { FileNode, ScanState, ViewMode, ScanEvent, ScanCache } from './lib/types';
  import { formatRelativeTime } from './lib/format';
  import Toolbar from './components/Toolbar.svelte';
  import ListView from './components/ListView.svelte';
  import TreemapView from './components/TreemapView.svelte';
  import SunburstView from './components/SunburstView.svelte';
  import StatusBar from './components/StatusBar.svelte';

  // --- State ---

  let scanState: ScanState = 'idle';
  let rootNodes: FileNode[] = [];
  let totalFileCount = 0;
  let totalSize = 0;
  let scanTime = 0;
  let viewMode: ViewMode = 'list';

  // --- Derived ---

  $: scanning = scanState === 'scanning';

  // --- Tree merge utility ---

  function mergeNodesIntoTree(existing: FileNode[], incoming: FileNode[]): FileNode[] {
    const map = new Map(existing.map(n => [n.id, n]));
    for (const node of incoming) {
      const existingNode = map.get(node.id);
      if (existingNode) {
        map.set(node.id, {
          ...existingNode,
          children: [...existingNode.children, ...node.children],
        });
      } else {
        map.set(node.id, node);
      }
    }
    return Array.from(map.values());
  }

  // --- Batched node buffer ---

  let pendingNodes: FileNode[] = [];
  let flushScheduled = false;

  function scheduleFlush() {
    if (!flushScheduled) {
      flushScheduled = true;
      requestAnimationFrame(() => {
        if (pendingNodes.length > 0) {
          rootNodes = mergeNodesIntoTree(rootNodes, pendingNodes);
          pendingNodes = [];
        }
        flushScheduled = false;
      });
    }
  }

  // --- Methods ---

  async function startScan(): Promise<void> {
    scanState = 'scanning';
    rootNodes = [];
    pendingNodes = [];
    totalFileCount = 0;
    totalSize = 0;

    try {
      await invoke('scan', { mode: 'default' });
    } catch (err) {
      console.error('Scan failed:', err);
      scanState = 'idle';
    }
  }

  async function cancelScan(): Promise<void> {
    try {
      await invoke('cancel_scan');
      scanState = 'idle';
    } catch (err) {
      console.error('Cancel failed:', err);
    }
  }

  function switchView(mode: ViewMode): void {
    viewMode = mode;
  }

  async function openFile(path: string): Promise<void> {
    try {
      await invoke('open_in_explorer', { path });
    } catch (err) {
      console.error('Failed to open file:', err);
    }
  }

  // --- Lifecycle ---

  onMount(async () => {
    const unlistenProgress = await listen<ScanEvent>('scan_progress', (event) => {
      const payload = event.payload;
      if (payload && payload.nodes) {
        for (const node of payload.nodes) {
          pendingNodes.push(node);
        }
        scheduleFlush();
      }
    });

    const unlistenComplete = await listen<ScanCache>('scan_complete', (event) => {
      const payload = event.payload;
      if (payload) {
        totalFileCount = payload.totalFileCount;
        totalSize = payload.totalSize;
        scanTime = payload.scanTime;
      }
      // Flush any remaining pending nodes
      if (pendingNodes.length > 0) {
        rootNodes = mergeNodesIntoTree(rootNodes, pendingNodes);
        pendingNodes = [];
      }
      scanState = 'done';
    });

    const unlistenCancelled = await listen('scan_cancelled', () => {
      if (pendingNodes.length > 0) {
        rootNodes = mergeNodesIntoTree(rootNodes, pendingNodes);
        pendingNodes = [];
      }
      scanState = 'idle';
    });

    // Check for existing cache
    try {
      const hasCache: boolean = await invoke('check_cache');
      if (hasCache) {
        const cache: ScanCache = await invoke('load_cache');
        rootNodes = cache.rootNodes;
        totalFileCount = cache.totalFileCount;
        totalSize = cache.totalSize;
        scanTime = cache.scanTime;
        scanState = 'done';
      } else {
        await startScan();
      }
    } catch (err) {
      console.error('Cache check failed:', err);
      await startScan();
    }

    return () => {
      unlistenProgress();
      unlistenComplete();
      unlistenCancelled();
    };
  });
</script>

<main>
  <Toolbar
    {scanState}
    {viewMode}
    onScan={startScan}
    onSwitchView={switchView}
    onCancel={cancelScan}
  />

  <div class="content">
    {#if viewMode === 'list'}
      <ListView {rootNodes} {scanning} onOpenFile={openFile} />
    {:else if viewMode === 'treemap'}
      <TreemapView {rootNodes} onOpenFile={openFile} />
    {:else if viewMode === 'sunburst'}
      <SunburstView {rootNodes} onOpenFile={openFile} />
    {/if}
  </div>

  <StatusBar {scanState} {totalFileCount} {totalSize} {scanTime} />
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: var(--font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif);
    background: var(--bg-body, #fafafa);
    color: var(--text-primary, #1a1a2e);
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-body, #fafafa);
  }

  .content {
    flex: 1;
    overflow: auto;
  }
</style>
