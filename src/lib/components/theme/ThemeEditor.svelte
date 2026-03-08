<script lang="ts">
  import { activeTheme, applyTheme, userThemes } from '$lib/stores/theme';
  import { addToast } from '$lib/stores/ui';
  import * as tauriApi from '$lib/utils/tauri';
  import type { Theme } from '$lib/types';

  let themeName = $state('Custom');
  let colors = $state<Record<string, string>>({ ...$activeTheme.colors });

  const colorKeys = [
    { key: 'bg-primary', label: 'Background' },
    { key: 'bg-secondary', label: 'Secondary BG' },
    { key: 'bg-surface', label: 'Surface' },
    { key: 'text-primary', label: 'Text' },
    { key: 'text-secondary', label: 'Text Secondary' },
    { key: 'accent', label: 'Accent' },
    { key: 'accent-secondary', label: 'Accent 2' },
    { key: 'danger', label: 'Danger' },
    { key: 'success', label: 'Success' },
    { key: 'warning', label: 'Warning' },
    { key: 'border', label: 'Border' },
    { key: 'diff-added-bg', label: 'Diff Added BG' },
    { key: 'diff-removed-bg', label: 'Diff Removed BG' },
    { key: 'diff-added-text', label: 'Diff Added Text' },
    { key: 'diff-removed-text', label: 'Diff Removed Text' },
  ];

  function preview() {
    const theme: Theme = { name: themeName, colors: { ...colors } };
    applyTheme(theme);
  }

  async function save() {
    const theme: Theme = { name: themeName, colors: { ...colors } };
    try {
      await tauriApi.saveUserTheme(theme);
      $userThemes = [...$userThemes.filter((t) => t.name !== themeName), theme];
      applyTheme(theme);
      addToast(`Theme '${themeName}' saved`, 'success');
    } catch (e) {
      addToast(`Failed to save theme: ${e}`, 'error');
    }
  }
</script>

<div class="theme-editor">
  <input
    class="name-input"
    type="text"
    placeholder="Theme name..."
    bind:value={themeName}
  />
  <div class="color-grid">
    {#each colorKeys as { key, label }}
      <label class="color-row">
        <span class="color-label">{label}</span>
        <input
          type="color"
          class="color-picker"
          bind:value={colors[key]}
          oninput={preview}
        />
      </label>
    {/each}
  </div>
  <div class="editor-actions">
    <button class="save-btn" onclick={save}>Save Theme</button>
  </div>
</div>

<style>
  .theme-editor {
    padding: 10px;
    border-top: 1px solid var(--border);
    max-height: 400px;
    overflow-y: auto;
  }
  .name-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    margin-bottom: 10px;
    box-sizing: border-box;
  }
  .color-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .color-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .color-picker {
    width: 28px;
    height: 22px;
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
    background: none;
  }
  .editor-actions {
    margin-top: 10px;
  }
  .save-btn {
    width: 100%;
    padding: 8px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }
  .save-btn:hover {
    filter: brightness(1.1);
  }
</style>
