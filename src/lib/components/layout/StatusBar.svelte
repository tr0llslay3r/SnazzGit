<script lang="ts">
  import { repoInfo, workingStatus, currentBranch } from '$lib/stores/repo';

  let stagedCount = $derived($workingStatus?.staged.length ?? 0);
  let unstagedCount = $derived(($workingStatus?.unstaged.length ?? 0) + ($workingStatus?.untracked.length ?? 0));
</script>

<footer class="status-bar">
  {#if $repoInfo}
    <div class="status-left">
      <span class="branch-indicator">
        <svg viewBox="0 0 16 16" width="12" height="12"><path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z" fill="currentColor"/></svg>
        {$currentBranch ?? 'detached'}
      </span>
    </div>
    <div class="status-right">
      {#if stagedCount > 0}
        <span class="status-item staged">{stagedCount} staged</span>
      {/if}
      {#if unstagedCount > 0}
        <span class="status-item unstaged">{unstagedCount} changed</span>
      {/if}
      <span class="status-item path" title={$repoInfo.path}>{$repoInfo.path}</span>
    </div>
  {:else}
    <div class="status-left">
      <span class="status-item">No repository open</span>
    </div>
  {/if}
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 24px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 12px;
    flex-shrink: 0;
  }
  .status-left, .status-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .branch-indicator {
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 600;
  }
  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .status-item.staged {
    font-weight: 600;
  }
  .status-item.path {
    opacity: 0.7;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
