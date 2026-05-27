<script lang="ts">
  import type { ScanState, ViewMode } from '../lib/types';

  export let scanState: ScanState;
  export let viewMode: ViewMode;
  export let onScan: () => void;
  export let onSwitchView: (mode: ViewMode) => void;
  export let onCancel: () => void = () => {};

  function scanLabel(): string {
    switch (scanState) {
      case 'idle':
        return '开始扫描';
      case 'scanning':
        return '扫描中...';
      case 'done':
        return '重新扫描';
    }
  }

  const views: { mode: ViewMode; label: string; icon: string }[] = [
    { mode: 'list', label: '列表', icon: '☰' },
    { mode: 'treemap', label: '矩形树图', icon: '▦' },
    { mode: 'sunburst', label: '旭日图', icon: '◎' },
  ];
</script>

<header class="toolbar">
  <span class="brand">DiskPeek</span>

  <div class="scan-area">
    {#if scanState === 'scanning'}
      <button class="scan-btn scanning" disabled>扫描中...</button>
      <button class="cancel-btn" on:click={onCancel} title="取消扫描">✕</button>
    {:else}
      <button class="scan-btn" on:click={onScan}>{scanLabel()}</button>
    {/if}
  </div>

  <div class="view-switcher">
    {#each views as v}
      <button
        class="view-btn"
        class:active={viewMode === v.mode}
        on:click={() => onSwitchView(v.mode)}
        title={v.label}
      >
        {v.icon}
      </button>
    {/each}
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px;
    background: var(--bg-toolbar, #1e1e2e);
    color: #fff;
    flex-shrink: 0;
  }

  .brand {
    font-weight: 700;
    font-size: 16px;
    letter-spacing: 0.5px;
  }

  .scan-area {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .scan-btn {
    padding: 6px 20px;
    border: none;
    border-radius: 8px;
    background: var(--accent, #7c3aed);
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .scan-btn:hover:not(:disabled) {
    background: var(--accent-hover, #6d28d9);
  }

  .scan-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .scan-btn.scanning {
    opacity: 0.7;
    cursor: wait;
  }

  .cancel-btn {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: rgba(255, 80, 80, 0.2);
    color: #ff6b6b;
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
  }

  .cancel-btn:hover {
    background: rgba(255, 80, 80, 0.4);
  }

  .view-switcher {
    display: flex;
    gap: 4px;
  }

  .view-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: #ccc;
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }

  .view-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .view-btn.active {
    background: var(--accent, #7c3aed);
    color: #fff;
  }
</style>
