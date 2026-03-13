<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { showSearch, showBranchDialog, showStagingArea, showStashDialog, showTagDialog, showCompareDialog, selectedCommit, fileHistoryPath, compareRefs, addToast } from '$lib/stores/ui';
  import { repoInfo, workingStatus, refreshCommits, refreshStatus, loadRecentRepos } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import SearchBar from '$lib/components/shared/SearchBar.svelte';
  import Toast from '$lib/components/shared/Toast.svelte';
  import ContextMenu from '$lib/components/shared/ContextMenu.svelte';

  let { children } = $props();

  onMount(async () => {
    loadSavedTheme();
    await loadRecentRepos();
    try {
      const args = await tauri.getCliArgs();
      if (args.length > 0) {
        const path = args[0];
        const info = await tauri.openRepository(path);
        $repoInfo = info;
        await tauri.addRecentRepo(info.path, info.name);
        await loadRecentRepos();
        await Promise.all([refreshCommits(), refreshStatus()]);
      }
    } catch (e) {
      addToast(`Failed to open repository: ${e}`, 'error');
    }
  });

  async function stageAllShortcut() {
    if (!$repoInfo) return;
    try {
      await tauri.stageAll($repoInfo.path);
      $workingStatus = await tauri.getStatus($repoInfo.path);
      addToast('Staged all files', 'success');
    } catch (e) {
      addToast(`Stage all failed: ${e}`, 'error');
    }
  }

  async function unstageAllShortcut() {
    if (!$repoInfo) return;
    try {
      await tauri.unstageAll($repoInfo.path);
      $workingStatus = await tauri.getStatus($repoInfo.path);
      addToast('Unstaged all files', 'success');
    } catch (e) {
      addToast(`Unstage all failed: ${e}`, 'error');
    }
  }

  function onKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const isEditable = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      $showSearch = !$showSearch;
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
      e.preventDefault();
      $showBranchDialog = !$showBranchDialog;
    }
    // Ctrl+Shift+S: stage all
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'S') {
      e.preventDefault();
      stageAllShortcut();
    }
    // Ctrl+Shift+U: unstage all
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'U') {
      e.preventDefault();
      unstageAllShortcut();
    }
    // Ctrl+T: toggle staging area
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 't') {
      e.preventDefault();
      $showStagingArea = !$showStagingArea;
    }
    // Ctrl+G: stash dialog
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'g') {
      e.preventDefault();
      $showStashDialog = !$showStashDialog;
    }
    // Ctrl+D: compare refs dialog
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'd') {
      e.preventDefault();
      $showCompareDialog = !$showCompareDialog;
    }
    // Escape: close file history, compare, deselect commit, close search
    if (e.key === 'Escape' && !isEditable) {
      if ($compareRefs) {
        $compareRefs = null;
      } else if ($fileHistoryPath) {
        $fileHistoryPath = null;
      } else if ($selectedCommit) {
        $selectedCommit = null;
      } else if ($showSearch) {
        $showSearch = false;
      }
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
      if (!isEditable) {
        e.preventDefault();
      }
    }
  }
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={(e) => e.preventDefault()} />

<ContextMenu />
<SearchBar />
<Toast />
{@render children()}
