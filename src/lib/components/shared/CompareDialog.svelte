<script lang="ts">
  import { repoInfo } from '$lib/stores/repo';
  import { showCompareDialog, compareRefs, addToast } from '$lib/stores/ui';

  let fromRef = $state('');
  let toRef = $state('');

  let allRefs = $derived(() => {
    if (!$repoInfo) return [];
    const refs: string[] = [];
    for (const b of $repoInfo.branches) {
      refs.push(b.name);
    }
    for (const t of $repoInfo.tags) {
      refs.push(t);
    }
    return refs;
  });

  function submit() {
    if (!fromRef.trim() || !toRef.trim()) {
      addToast('Please specify both refs', 'warning');
      return;
    }
    $compareRefs = { from: fromRef.trim(), to: toRef.trim() };
    $showCompareDialog = false;
    fromRef = '';
    toRef = '';
  }

  function close() {
    $showCompareDialog = false;
    fromRef = '';
    toRef = '';
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

{#if $showCompareDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown}>
      <h3>Compare Refs</h3>
      <label class="field-label">
        From (base)
        <!-- svelte-ignore a11y_autofocus -->
        <input type="text" bind:value={fromRef} list="ref-options" placeholder="branch, tag, or commit hash" autofocus />
      </label>
      <label class="field-label">
        To
        <input type="text" bind:value={toRef} list="ref-options" placeholder="branch, tag, or commit hash" />
      </label>
      <datalist id="ref-options">
        {#each allRefs() as ref}
          <option value={ref}></option>
        {/each}
      </datalist>
      <div class="dialog-actions">
        <button class="btn-cancel" onclick={close}>Cancel</button>
        <button class="btn-primary" onclick={submit}>Compare</button>
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
    min-width: 380px;
    max-width: 480px;
  }
  h3 {
    margin: 0 0 16px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .field-label {
    display: block;
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 12px;
  }
  .field-label input {
    display: block;
    width: 100%;
    margin-top: 4px;
    padding: 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    box-sizing: border-box;
  }
  .field-label input:focus {
    border-color: var(--accent);
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
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
  .btn-primary:hover {
    filter: brightness(1.1);
  }
</style>
