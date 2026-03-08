<script lang="ts">
  interface Props {
    path: string;
    status: string;
    selected?: boolean;
    onSelect?: () => void;
    onAction?: () => void;
    actionLabel?: string;
    actionIcon?: string;
    onContextMenu?: (e: MouseEvent) => void;
  }

  let { path, status, selected = false, onSelect, onAction, actionLabel = '', actionIcon = '', onContextMenu }: Props = $props();

  let statusClass = $derived(
    status === 'New' ? 'status-new' :
    status === 'Modified' ? 'status-modified' :
    status === 'Deleted' ? 'status-deleted' :
    status === 'Renamed' ? 'status-renamed' :
    status === 'Conflicted' ? 'status-conflicted' : ''
  );

  let statusLetter = $derived(
    status === 'New' ? 'A' :
    status === 'Modified' ? 'M' :
    status === 'Deleted' ? 'D' :
    status === 'Renamed' ? 'R' :
    status === 'Conflicted' ? 'C' :
    status === 'untracked' ? '?' : 'M'
  );

  let filename = $derived(path.split('/').pop() ?? path);
  let dirname = $derived(path.includes('/') ? path.slice(0, path.lastIndexOf('/') + 1) : '');
</script>

<div class="file-entry" class:selected role="listitem" oncontextmenu={(e) => { if (onContextMenu) { e.preventDefault(); e.stopPropagation(); onContextMenu(e); } }}>
  <button class="file-info" onclick={onSelect}>
    <span class="status-badge {statusClass}">{statusLetter}</span>
    <span class="file-path">
      {#if dirname}
        <span class="dirname">{dirname}</span>
      {/if}
      <span class="filename">{filename}</span>
    </span>
  </button>
  {#if onAction}
    <button class="file-action" onclick={onAction} title={actionLabel}>
      {actionIcon || actionLabel}
    </button>
  {/if}
</div>

<style>
  .file-entry {
    display: flex;
    align-items: center;
    padding: 0 18px 0 8px;
    height: 26px;
    gap: 4px;
  }
  .file-entry:hover {
    background: var(--bg-surface);
  }
  .file-entry.selected {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }
  .file-info {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    overflow: hidden;
  }
  .status-badge {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 700;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .status-new { background: color-mix(in srgb, var(--success) 25%, transparent); color: var(--success); }
  .status-modified { background: color-mix(in srgb, var(--warning) 25%, transparent); color: var(--warning); }
  .status-deleted { background: color-mix(in srgb, var(--danger) 25%, transparent); color: var(--danger); }
  .status-renamed { background: color-mix(in srgb, var(--accent) 25%, transparent); color: var(--accent); }
  .status-conflicted { background: color-mix(in srgb, var(--danger) 40%, transparent); color: var(--danger); }
  .file-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dirname {
    color: var(--text-secondary);
  }
  .file-action {
    display: none;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    background: var(--bg-surface);
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
    flex-shrink: 0;
  }
  .file-entry:hover .file-action {
    display: flex;
  }
  .file-action:hover {
    color: var(--text-primary);
    background: var(--border);
  }
</style>
