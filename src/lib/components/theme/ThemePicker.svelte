<script lang="ts">
  import { showThemePicker } from '$lib/stores/ui';
  import { builtinThemes, userThemes, activeTheme, applyTheme } from '$lib/stores/theme';
  import type { Theme } from '$lib/types';

  let showEditor = $state(false);

  function selectTheme(theme: Theme) {
    applyTheme(theme);
  }

  let allThemes = $derived([...builtinThemes, ...$userThemes]);
</script>

{#if $showThemePicker}
  <div class="theme-panel">
    <div class="theme-header">
      <h3>Themes</h3>
      <button class="close-btn" onclick={() => $showThemePicker = false} aria-label="Close theme picker">
        <svg viewBox="0 0 16 16" width="14" height="14"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z" fill="currentColor"/></svg>
      </button>
    </div>
    <div class="theme-list">
      {#each allThemes as theme (theme.name)}
        <button
          class="theme-item"
          class:active={$activeTheme.name === theme.name}
          onclick={() => selectTheme(theme)}
        >
          <div class="theme-preview">
            <div class="preview-dot" style="background: {theme.colors['bg-primary']}"></div>
            <div class="preview-dot" style="background: {theme.colors['accent']}"></div>
            <div class="preview-dot" style="background: {theme.colors['accent-secondary']}"></div>
            <div class="preview-dot" style="background: {theme.colors['success']}"></div>
          </div>
          <span class="theme-name">{theme.name}</span>
          {#if $activeTheme.name === theme.name}
            <svg viewBox="0 0 16 16" width="14" height="14" class="check"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0z" fill="currentColor"/></svg>
          {/if}
        </button>
      {/each}
    </div>
    <button class="editor-btn" onclick={() => showEditor = !showEditor}>
      {showEditor ? 'Close Editor' : 'Create Custom Theme'}
    </button>
    {#if showEditor}
      {@const EditorPromise = import('./ThemeEditor.svelte')}
      {#await EditorPromise then module}
        <module.default />
      {/await}
    {/if}
  </div>
{/if}

<style>
  .theme-panel {
    position: fixed;
    top: 48px;
    right: 12px;
    width: 280px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    z-index: 50;
    overflow: hidden;
  }
  .theme-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .theme-header h3 {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
  }
  .close-btn:hover {
    background: var(--bg-surface);
  }
  .theme-list {
    padding: 6px;
  }
  .theme-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    border-radius: 6px;
    font-size: 13px;
  }
  .theme-item:hover {
    background: var(--bg-surface);
  }
  .theme-item.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }
  .theme-preview {
    display: flex;
    gap: 3px;
  }
  .preview-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1px solid var(--border);
  }
  .theme-name {
    flex: 1;
    text-align: left;
  }
  .check {
    color: var(--accent);
  }
  .editor-btn {
    display: block;
    width: calc(100% - 12px);
    margin: 6px;
    padding: 8px;
    border: 1px dashed var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
  }
  .editor-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
