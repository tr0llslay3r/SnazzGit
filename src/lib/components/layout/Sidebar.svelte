<script lang="ts">
  import TreeView from '$lib/components/shared/TreeView.svelte';
  import { repoInfo, localBranches, remoteBranches, stashEntries, workingStatus, commits, refreshAll, refreshStatus, refreshStashes } from '$lib/stores/repo';
  import { showBranchDialog, showStashDialog, selectedCommit, showStagingArea, selectedFile, addToast, jumpToCommitId } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';

  $effect(() => {
    if ($repoInfo) {
      refreshStashes();
    }
  });

  function branchNodes() {
    return $localBranches.map((b) => ({
      label: b.name,
      icon: b.is_head ? '\u25CF' : '',
      data: b,
    }));
  }

  function remoteNodes() {
    const byRemote: Record<string, { label: string; data: unknown }[]> = {};
    for (const b of $remoteBranches) {
      const [remote, ...rest] = b.name.split('/');
      if (!byRemote[remote]) byRemote[remote] = [];
      byRemote[remote].push({ label: rest.join('/'), data: b });
    }
    return Object.entries(byRemote).map(([name, children]) => ({
      label: name,
      children,
    }));
  }

  function tagNodes() {
    return ($repoInfo?.tags ?? []).map((t) => ({ label: t }));
  }

  function stashNodes() {
    return $stashEntries.map((s) => ({
      label: s.message || `stash@{${s.index}}`,
      data: s,
    }));
  }

  let changeCount = $derived(
    ($workingStatus?.staged.length ?? 0) +
    ($workingStatus?.unstaged.length ?? 0) +
    ($workingStatus?.untracked.length ?? 0)
  );

  function showWorkingCopy() {
    $selectedCommit = null;
    $selectedFile = null;
    $showStagingArea = true;
  }

  function onBranchContext(node: { label: string; data?: unknown }, e: MouseEvent) {
    if (!$repoInfo) return;
    const branch = node.data as { name: string; is_head: boolean } | undefined;
    if (!branch) return;
    const items: ContextMenuEntry[] = [];
    if (branch.is_head) {
      items.push({ label: 'Push', action: async () => {
        try {
          await tauri.push($repoInfo!.path, 'origin');
          await refreshAll();
          addToast(`Pushed ${branch.name}`, 'success');
        } catch (err) { addToast(`Push failed: ${err}`, 'error'); }
      }});
    } else {
      items.push({ label: 'Checkout', action: async () => {
        try {
          await tauri.checkoutBranch($repoInfo!.path, branch.name);
          await refreshAll();
          addToast(`Switched to ${branch.name}`, 'success');
        } catch (err) { addToast(`Checkout failed: ${err}`, 'error'); }
      }});
    }
    items.push({ label: 'Merge into current', action: async () => {
      try {
        const msg = await tauri.mergeBranch($repoInfo!.path, branch.name);
        await refreshAll();
        addToast(msg, 'success');
      } catch (err) { addToast(`Merge failed: ${err}`, 'error'); }
    }});
    if (!branch.is_head) {
      items.push({ separator: true });
      items.push({ label: 'Rename', action: async () => {
        const newName = prompt(`Rename branch "${branch.name}" to:`);
        if (!newName || !newName.trim()) return;
        try {
          await tauri.renameBranch($repoInfo!.path, branch.name, newName.trim());
          await refreshAll();
          addToast(`Renamed to ${newName.trim()}`, 'success');
        } catch (err) { addToast(`Rename failed: ${err}`, 'error'); }
      }});
      items.push({ label: 'Delete', danger: true, action: async () => {
        try {
          await tauri.deleteBranch($repoInfo!.path, branch.name);
          await refreshAll();
          addToast(`Deleted ${branch.name}`, 'success');
        } catch (err) { addToast(`Delete failed: ${err}`, 'error'); }
      }});
    }
    showContextMenu(e.clientX, e.clientY, items);
  }

  function onStashContext(node: { label: string; data?: unknown }, e: MouseEvent) {
    if (!$repoInfo) return;
    const stash = node.data as { index: number; message: string } | undefined;
    if (!stash) return;
    const items: ContextMenuEntry[] = [
      { label: 'Apply', action: async () => {
        try {
          await tauri.stashApply($repoInfo!.path, stash.index);
          await refreshAll();
          addToast('Stash applied', 'success');
        } catch (err) { addToast(`Apply failed: ${err}`, 'error'); }
      }},
      { label: 'Pop', action: async () => {
        try {
          await tauri.stashPop($repoInfo!.path, stash.index);
          await refreshAll();
          addToast('Stash popped', 'success');
        } catch (err) { addToast(`Pop failed: ${err}`, 'error'); }
      }},
      { separator: true },
      { label: 'Drop', danger: true, action: async () => {
        try {
          await tauri.stashDrop($repoInfo!.path, stash.index);
          await refreshAll();
          addToast('Stash dropped', 'success');
        } catch (err) { addToast(`Drop failed: ${err}`, 'error'); }
      }},
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }

  function saveStash() {
    $showStashDialog = true;
  }

  function onTagSelect(node: { label: string; data?: unknown }) {
    const tagName = node.label;
    const commit = $commits.find((c) => c.refs.some((r) => r.ref_type === 'Tag' && r.name === tagName));
    if (commit) {
      $jumpToCommitId = commit.id;
    }
  }

  async function onBranchSelect(node: { label: string; data?: unknown }) {
    if (!$repoInfo) return;
    const branch = node.data as { name: string; is_head: boolean } | undefined;
    if (branch && !branch.is_head) {
      try {
        await tauri.checkoutBranch($repoInfo.path, branch.name);
        await refreshAll();
        addToast(`Switched to ${branch.name}`, 'success');
      } catch (e) {
        addToast(`Failed to checkout: ${e}`, 'error');
      }
    }
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    {#if $repoInfo}
      <h2 class="repo-name">{$repoInfo.name}</h2>
    {:else}
      <h2 class="repo-name">No Repository</h2>
    {/if}
  </div>

  {#if $repoInfo}
    <button
      class="working-copy-btn"
      class:active={!$selectedCommit && $showStagingArea}
      onclick={showWorkingCopy}
    >
      <span class="wc-icon">&#9998;</span>
      <span class="wc-label">Working Copy</span>
      {#if changeCount > 0}
        <span class="wc-badge">{changeCount}</span>
      {/if}
    </button>

    <div class="sidebar-section">
      <div class="section-header">
        <span>Branches</span>
        <button class="section-action" onclick={() => $showBranchDialog = true} title="New branch">+</button>
      </div>
      <TreeView nodes={branchNodes()} onDblSelect={onBranchSelect} onContextMenu={onBranchContext} />
    </div>

    <div class="sidebar-section">
      <div class="section-header"><span>Remotes</span></div>
      <TreeView nodes={remoteNodes()} />
    </div>

    <div class="sidebar-section">
      <div class="section-header"><span>Tags</span></div>
      <TreeView nodes={tagNodes()} onSelect={onTagSelect} />
    </div>

    <div class="sidebar-section">
      <div class="section-header">
        <span>Stashes</span>
        {#if $stashEntries.length > 0}
          <span class="count-badge">{$stashEntries.length}</span>
        {/if}
        <button class="section-action" onclick={saveStash} title="Stash changes">+</button>
      </div>
      <TreeView nodes={stashNodes()} onContextMenu={onStashContext} />
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    overflow-x: hidden;
    height: 100%;
  }
  .sidebar-header {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .repo-name {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .working-copy-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 14px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }
  .working-copy-btn:hover {
    background: var(--bg-surface);
  }
  .working-copy-btn.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    color: var(--accent);
  }
  .wc-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
  .wc-label {
    flex: 1;
    font-weight: 600;
  }
  .wc-badge {
    font-size: 10px;
    padding: 0 6px;
    border-radius: 8px;
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
  }
  .sidebar-section {
    padding: 6px 0;
  }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 14px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
    letter-spacing: 0.5px;
  }
  .section-action {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 0 4px;
    border-radius: 4px;
  }
  .section-action:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }
  .count-badge {
    font-size: 10px;
    padding: 0 5px;
    border-radius: 8px;
    background: var(--bg-surface);
    color: var(--text-secondary);
  }
</style>
