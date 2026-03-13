<script lang="ts">
  import TreeView from '$lib/components/shared/TreeView.svelte';
  import { repoInfo, localBranches, remoteBranches, stashEntries, workingStatus, commits, refreshAll, refreshStatus, refreshStashes } from '$lib/stores/repo';
  import { showBranchDialog, showStashDialog, selectedCommit, selectedFile, addToast, jumpToCommitId, showCheckoutRemoteDialog, checkoutRemoteBranch, showTagDialog, tagTargetCommitId, compareRefs, showReflog, showAddRemoteDialog } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';

  $effect(() => {
    if ($repoInfo) {
      refreshStashes();
    }
  });

  interface TreeNode {
    label: string;
    icon?: string;
    children?: TreeNode[];
    data?: unknown;
  }

  function buildTree(branches: typeof $localBranches, iconFn?: (b: typeof $localBranches[0]) => string): TreeNode[] {
    const root: TreeNode[] = [];
    const folders = new Map<string, TreeNode>();

    for (const b of branches) {
      const parts = b.name.split('/');
      if (parts.length === 1) {
        root.push({ label: b.name, icon: iconFn?.(b) ?? '', data: b });
        continue;
      }

      let parentList = root;
      let pathSoFar = '';
      for (let i = 0; i < parts.length - 1; i++) {
        pathSoFar += (pathSoFar ? '/' : '') + parts[i];
        let folder = folders.get(pathSoFar);
        if (!folder) {
          folder = { label: parts[i], children: [] };
          folders.set(pathSoFar, folder);
          parentList.push(folder);
        }
        parentList = folder.children!;
      }
      parentList.push({ label: parts[parts.length - 1], icon: iconFn?.(b) ?? '', data: b });
    }
    return root;
  }

  function branchNodes() {
    return buildTree($localBranches, (b) => b.is_head ? '\u25CF' : '');
  }

  function remoteNodes() {
    const byRemote: Record<string, typeof $remoteBranches> = {};
    for (const b of $remoteBranches) {
      const [remote, ...rest] = b.name.split('/');
      if (!byRemote[remote]) byRemote[remote] = [];
      byRemote[remote].push({ ...b, name: rest.join('/'), full_name: b.name });
    }
    return Object.entries(byRemote).map(([name, branches]) => ({
      label: name,
      children: buildTree(branches),
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
    $compareRefs = null;
    $showReflog = false;
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
    items.push({ label: 'Rebase onto...', action: async () => {
      try {
        const msg = await tauri.rebaseOnto($repoInfo!.path, branch.name);
        await refreshAll();
        if (msg.includes('conflicts')) {
          addToast(msg, 'warning');
        } else {
          addToast(msg, 'success');
        }
      } catch (err) { addToast(`Rebase failed: ${err}`, 'error'); }
    }});
    items.push({ label: 'Set Upstream...', action: async () => {
      const upstream = prompt(`Set upstream for "${branch.name}" (e.g., origin/${branch.name}):\nLeave empty to unset.`);
      if (upstream === null) return;
      try {
        await tauri.setUpstream($repoInfo!.path, branch.name, upstream.trim() || undefined);
        await refreshAll();
        addToast(upstream.trim() ? `Upstream set to ${upstream.trim()}` : 'Upstream unset', 'success');
      } catch (err) { addToast(`Set upstream failed: ${err}`, 'error'); }
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
      items.push({ label: 'Delete', children: [
        { label: 'Delete (safe)', action: async () => {
          try {
            await tauri.deleteBranch($repoInfo!.path, branch.name);
            await refreshAll();
            addToast(`Deleted ${branch.name}`, 'success');
          } catch (err) { addToast(`Delete failed: ${err}`, 'error'); }
        }},
        { label: 'Force Delete', danger: true, action: async () => {
          try {
            await tauri.forceDeleteBranch($repoInfo!.path, branch.name);
            await refreshAll();
            addToast(`Force deleted ${branch.name}`, 'success');
          } catch (err) { addToast(`Force delete failed: ${err}`, 'error'); }
        }},
      ]});
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

  function onTagContext(node: { label: string; data?: unknown }, e: MouseEvent) {
    if (!$repoInfo) return;
    const items: ContextMenuEntry[] = [
      { label: 'Delete', danger: true, action: async () => {
        try {
          await tauri.deleteTag($repoInfo!.path, node.label);
          await refreshAll();
          addToast(`Deleted tag "${node.label}"`, 'success');
        } catch (err) { addToast(`Delete tag failed: ${err}`, 'error'); }
      }},
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }

  function onBranchNavigate(node: { label: string; data?: unknown }) {
    if (!node.data) return;
    const branch = node.data as { name: string; commit_id: string };
    if (branch.commit_id) {
      $jumpToCommitId = branch.commit_id;
    }
  }

  function onRemoteNavigate(node: { label: string; data?: unknown }) {
    if (!node.data) return;
    const branch = node.data as { commit_id: string };
    if (branch.commit_id) {
      $jumpToCommitId = branch.commit_id;
    }
  }

  function remoteNodeToBranch(data: unknown): { name: string; is_head: boolean; is_remote: boolean; upstream: string | null; commit_id: string } {
    const d = data as any;
    return { ...d, name: d.full_name ?? d.name };
  }

  function onRemoteSelect(node: { label: string; data?: unknown }) {
    if (!node.data) return;
    $checkoutRemoteBranch = remoteNodeToBranch(node.data);
    $showCheckoutRemoteDialog = true;
  }

  function onRemoteContext(node: { label: string; data?: unknown }, e: MouseEvent) {
    if (!$repoInfo) return;
    const items: ContextMenuEntry[] = [];

    if (node.data) {
      // Branch node under a remote
      items.push({ label: 'Checkout...', action: () => {
        $checkoutRemoteBranch = remoteNodeToBranch(node.data!);
        $showCheckoutRemoteDialog = true;
      }});
    } else {
      // Remote group node (e.g., "origin")
      items.push({ label: 'Fetch', action: async () => {
        try {
          await tauri.fetchRemote($repoInfo!.path, node.label);
          await refreshAll();
          addToast(`Fetched ${node.label}`, 'success');
        } catch (err) { addToast(`Fetch failed: ${err}`, 'error'); }
      }});
      items.push({ separator: true });
      items.push({ label: 'Rename', action: async () => {
        const newName = prompt(`Rename remote "${node.label}" to:`);
        if (!newName || !newName.trim()) return;
        try {
          await tauri.renameRemote($repoInfo!.path, node.label, newName.trim());
          await refreshAll();
          addToast(`Renamed to ${newName.trim()}`, 'success');
        } catch (err) { addToast(`Rename failed: ${err}`, 'error'); }
      }});
      items.push({ label: 'Remove', danger: true, action: async () => {
        try {
          await tauri.removeRemote($repoInfo!.path, node.label);
          await refreshAll();
          addToast(`Removed remote "${node.label}"`, 'success');
        } catch (err) { addToast(`Remove failed: ${err}`, 'error'); }
      }});
    }
    showContextMenu(e.clientX, e.clientY, items);
  }

  function addRemoteDialog() {
    $showAddRemoteDialog = true;
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
      class:active={!$selectedCommit}
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
      <TreeView nodes={branchNodes()} onSelect={onBranchNavigate} onDblSelect={onBranchSelect} onContextMenu={onBranchContext} defaultExpanded={true} />
    </div>

    <div class="sidebar-section">
      <div class="section-header">
        <span>Remotes</span>
        <button class="section-action" onclick={addRemoteDialog} title="Add remote">+</button>
      </div>
      <TreeView nodes={remoteNodes()} onSelect={onRemoteNavigate} onDblSelect={onRemoteSelect} onContextMenu={onRemoteContext} />
    </div>

    <div class="sidebar-section">
      <div class="section-header">
        <span>Tags</span>
        <button class="section-action" onclick={() => $showTagDialog = true} title="New tag">+</button>
      </div>
      <TreeView nodes={tagNodes()} onSelect={onTagSelect} onContextMenu={onTagContext} />
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
