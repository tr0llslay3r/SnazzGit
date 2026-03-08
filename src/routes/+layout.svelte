<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { showSearch, showBranchDialog, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshCommits, refreshStatus, loadRecentRepos } from '$lib/stores/repo';
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

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      $showSearch = !$showSearch;
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
      e.preventDefault();
      $showBranchDialog = !$showBranchDialog;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={(e) => e.preventDefault()} />

<ContextMenu />
<SearchBar />
<Toast />
{@render children()}
