<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { FileNode, ScanState, ViewMode, ScanEvent, ScanCache } from './lib/types';
  import { formatRelativeTime } from './lib/format';
  import Toolbar from './components/Toolbar.svelte';
  import ListView from './components/ListView.svelte';
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

  // --- Methods ---

  function flattenNodes(nodes: FileNode[]): void {
    for (const node of nodes) {
      rootNodes.push(node);
      if (node.children && node.children.length > 0) {
        flattenNodes(node.children);
      }
    }
  }

  async function startScan(): Promise<void> {
    scanState = 'scanning';
    rootNodes = [];
    totalFileCount = 0;
    totalSize = 0;

    try {
      await invoke('scan', { mode: 'default' });
    } catch (err) {
      console.error('Scan failed:', err);
      scanState = 'idle';
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
    // Listen for scan events
    const unlistenProgress = await listen<ScanEvent>('scan_progress', (event) => {
      const payload = event.payload;
      if (payload && payload.nodes) {
        for (const node of payload.nodes) {
          rootNodes.push(node);
        }
      }
    });

    const unlistenComplete = await listen<ScanCache>('scan_complete', (event) => {
      const payload = event.payload;
      if (payload) {
        totalFileCount = payload.totalFileCount;
        totalSize = payload.totalSize;
        scanTime = payload.scanTime;
      }
      scanState = 'done';
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
    };
  });
</script>

<main>
  <Toolbar
    {scanState}
    {viewMode}
    onScan={startScan}
    onSwitchView={switchView}
  />

  <div class="content">
    {#if viewMode === 'list'}
      <ListView {rootNodes} {scanning} {onOpenFile} />
    {:else if viewMode === 'treemap'}
      <div class="placeholder">Treemap 视图开发中...</div>
    {:else if viewMode === 'sunburst'}
      <div class="placeholder">Sunburst 视图开发中...</div>
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
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: #fafafa;
    color: #1a1a2e;
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #fafafa;
  }

  .content {
    flex: 1;
    overflow: auto;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 18px;
    color: #888;
  }
</style>