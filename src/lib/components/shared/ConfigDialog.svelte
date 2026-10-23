<script lang="ts">
  import { showConfigDialog, addToast } from '$lib/stores/ui';
  import { repoInfo } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import type { ConfigEntry } from '$lib/types';

  let entries = $state<ConfigEntry[]>([]);
  let loading = $state(false);
  let newKey = $state('');
  let newValue = $state('');

  async function load() {
    if (!$repoInfo) return;
    loading = true;
    try {
      entries = await tauri.getConfigEntries($repoInfo.path);
    } catch (e) {
      addToast(`Failed to load config: ${e}`, 'error');
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if ($showConfigDialog) {
      load();
    }
  });

  async function saveEntry(entry: ConfigEntry) {
    if (!$repoInfo) return;
    try {
      await tauri.setConfigEntry($repoInfo.path, entry.name, entry.value);
      addToast(`Updated ${entry.name}`, 'success');
    } catch (e) {
      addToast(`Failed to update ${entry.name}: ${e}`, 'error');
      await load();
    }
  }

  async function deleteEntry(entry: ConfigEntry) {
    if (!$repoInfo) return;
    try {
      await tauri.deleteConfigEntry($repoInfo.path, entry.name);
      entries = entries.filter((e) => e.name !== entry.name);
      addToast(`Removed ${entry.name}`, 'success');
    } catch (e) {
      addToast(`Failed to remove ${entry.name}: ${e}`, 'error');
    }
  }

  async function addEntry() {
    if (!$repoInfo || !newKey.trim()) return;
    const key = newKey.trim();
    const value = newValue;
    try {
      await tauri.setConfigEntry($repoInfo.path, key, value);
      newKey = '';
      newValue = '';
      await load();
      addToast(`Added ${key}`, 'success');
    } catch (e) {
      addToast(`Failed to add ${key}: ${e}`, 'error');
    }
  }

  function close() {
    $showConfigDialog = false;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

{#if $showConfigDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown}>
      <h3>Repository Config</h3>
      <p class="hint">Editing this repository's local config (<code>.git/config</code>) only.</p>

      <div class="entry-list">
        {#if loading}
          <div class="empty-state">Loading...</div>
        {:else if entries.length === 0}
          <div class="empty-state">No config entries</div>
        {:else}
          {#each entries as entry (entry.name)}
            <div class="entry-row">
              <span class="entry-name" title={entry.name}>{entry.name}</span>
              <input
                type="text"
                class="entry-value"
                bind:value={entry.value}
                onblur={() => saveEntry(entry)}
                onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
              />
              <button class="entry-delete" title="Remove" onclick={() => deleteEntry(entry)}>&times;</button>
            </div>
          {/each}
        {/if}
      </div>

      <div class="add-row">
        <input type="text" class="add-key" placeholder="key (e.g. user.name)" bind:value={newKey} />
        <input type="text" class="add-value" placeholder="value" bind:value={newValue}
          onkeydown={(e) => { if (e.key === 'Enter') addEntry(); }} />
        <button class="btn-primary" onclick={addEntry} disabled={!newKey.trim()}>Add</button>
      </div>

      <div class="dialog-actions">
        <button class="btn-cancel" onclick={close}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    width: 520px;
    max-width: 90vw;
  }
  h3 {
    margin: 0 0 4px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .hint {
    margin: 0 0 14px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .hint code {
    background: var(--bg-surface);
    padding: 1px 4px;
    border-radius: 4px;
  }
  .entry-list {
    max-height: 320px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
  }
  .empty-state {
    padding: 16px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .entry-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
  }
  .entry-row:hover {
    background: var(--bg-surface);
    border-radius: 6px;
  }
  .entry-name {
    flex: 0 0 40%;
    font-size: 12px;
    font-family: monospace;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-value {
    flex: 1;
    min-width: 0;
    padding: 4px 6px;
    background: var(--bg-primary);
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: monospace;
    outline: none;
  }
  .entry-value:focus {
    border-color: var(--accent);
  }
  .entry-delete {
    flex-shrink: 0;
    border: none;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .entry-delete:hover {
    background: var(--danger);
    color: white;
  }
  .add-row {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .add-key {
    flex: 0 0 40%;
  }
  .add-value {
    flex: 1;
    min-width: 0;
  }
  .add-row input {
    padding: 6px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
  }
  .add-row input:focus {
    border-color: var(--accent);
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
  }
  .btn-cancel {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
  }
  .btn-primary:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }
</style>
