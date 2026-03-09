<script lang="ts">
  import { showSearch, searchQuery, jumpToCommitId } from '$lib/stores/ui';
  import { repoInfo } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import type { CommitInfo } from '$lib/types';

  let input: HTMLInputElement = $state(null!);
  let results: CommitInfo[] = $state([]);
  let selectedIndex = $state(0);
  let searching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if ($showSearch && input) {
      results = [];
      selectedIndex = 0;
      input.focus();
    }
  });

  $effect(() => {
    const query = $searchQuery;
    clearTimeout(debounceTimer);
    if (!query.trim() || !$repoInfo) {
      results = [];
      selectedIndex = 0;
      return;
    }
    debounceTimer = setTimeout(async () => {
      searching = true;
      try {
        results = await tauri.searchCommits($repoInfo!.path, query.trim(), 20);
        selectedIndex = 0;
      } catch {
        results = [];
      }
      searching = false;
    }, 200);
  });

  function close() {
    $showSearch = false;
    $searchQuery = '';
    results = [];
  }

  function selectResult(commit: CommitInfo) {
    $jumpToCommitId = commit.id;
    close();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (results.length > 0) {
        selectedIndex = (selectedIndex + 1) % results.length;
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (results.length > 0) {
        selectedIndex = (selectedIndex - 1 + results.length) % results.length;
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results.length > 0 && results[selectedIndex]) {
        selectResult(results[selectedIndex]);
      }
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      year: 'numeric', month: 'short', day: 'numeric',
    });
  }
</script>

{#if $showSearch}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="search-backdrop" onclick={close}></div>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="search-overlay" onkeydown={onKeydown} role="search">
    <div class="search-panel">
      <div class="search-box">
        <svg viewBox="0 0 16 16" width="16" height="16" class="search-icon">
          <path d="M11.5 7a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zm-.82 4.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 1 1-1.06 1.06l-3.04-3.04z" fill="currentColor"/>
        </svg>
        <input
          bind:this={input}
          bind:value={$searchQuery}
          type="text"
          placeholder="Search commits by message, author, or SHA..."
          class="search-input"
        />
        {#if searching}
          <span class="search-spinner">…</span>
        {/if}
        <button class="search-close" onclick={close} aria-label="Close search">
          <svg viewBox="0 0 16 16" width="14" height="14">
            <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z" fill="currentColor"/>
          </svg>
        </button>
      </div>
      {#if $searchQuery.trim()}
        <div class="search-results">
          {#if results.length === 0 && !searching}
            <div class="search-empty">No commits found</div>
          {:else}
            {#each results as commit, i (commit.id)}
              <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
              <div
                class="search-result"
                class:selected={i === selectedIndex}
                onclick={() => selectResult(commit)}
                onmouseenter={() => selectedIndex = i}
              >
                <div class="result-top">
                  <span class="result-sha">{commit.short_id}</span>
                  <span class="result-date">{formatDate(commit.author_time)}</span>
                </div>
                <div class="result-message">{commit.summary}</div>
                <div class="result-author">{commit.author_name}</div>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }
  .search-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    padding-top: 80px;
    z-index: 100;
  }
  .search-panel {
    width: 520px;
    max-width: 90vw;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }
  .search-box {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
  }
  .search-icon {
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 15px;
    outline: none;
  }
  .search-input::placeholder {
    color: var(--text-secondary);
  }
  .search-spinner {
    color: var(--text-secondary);
    font-size: 14px;
    animation: pulse 1s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }
  .search-close {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
  }
  .search-close:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }
  .search-results {
    max-height: 400px;
    overflow-y: auto;
    padding: 4px;
  }
  .search-empty {
    padding: 20px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .search-result {
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
  }
  .search-result:hover,
  .search-result.selected {
    background: var(--bg-surface);
  }
  .result-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2px;
  }
  .result-sha {
    font-family: monospace;
    font-size: 11px;
    color: var(--accent);
  }
  .result-date {
    font-size: 11px;
    color: var(--text-secondary);
  }
  .result-message {
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .result-author {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 1px;
  }
</style>
