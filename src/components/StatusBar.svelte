<script lang="ts">
  import type { ScanState } from '../lib/types';
  import { formatSize, formatRelativeTime } from '../lib/format';

  export let scanState: ScanState;
  export let totalFileCount: number;
  export let totalSize: number;
  export let scanTime: number;
</script>

<footer class="status-bar">
  {#if scanState === 'scanning'}
    <span>正在扫描… 已发现 {totalFileCount} 个文件 · {formatSize(totalSize)}</span>
  {:else if scanState === 'done'}
    <span>扫描完成 · {formatRelativeTime(scanTime)} · {totalFileCount} 个文件 · {formatSize(totalSize)}</span>
  {:else}
    <span>等待扫描</span>
  {/if}
</footer>

<style>
  .status-bar {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    height: 32px;
    display: flex;
    align-items: center;
    padding: 0 16px;
    background: #f0f0f0;
    font-size: 13px;
    color: #555;
    flex-shrink: 0;
    z-index: 10;
  }
</style>