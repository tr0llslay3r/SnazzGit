<script lang="ts">
  import { contextMenu, hideContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';

  let menuEl: HTMLDivElement | undefined = $state();
  let openSubmenu: string | null = $state(null);

  $effect(() => {
    if ($contextMenu.visible && menuEl) {
      openSubmenu = null;
      requestAnimationFrame(() => {
        if (!menuEl) return;
        const rect = menuEl.getBoundingClientRect();
        let x = $contextMenu.x;
        let y = $contextMenu.y;
        if (x + rect.width > window.innerWidth) x = window.innerWidth - rect.width - 4;
        if (y + rect.height > window.innerHeight) y = window.innerHeight - rect.height - 4;
        if (x < 0) x = 4;
        if (y < 0) y = 4;
        menuEl.style.left = `${x}px`;
        menuEl.style.top = `${y}px`;
      });
    }
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && $contextMenu.visible) {
      e.preventDefault();
      hideContextMenu();
    }
  }

  function handleClick(item: ContextMenuEntry) {
    if ('separator' in item || 'children' in item) return;
    hideContextMenu();
    item.action();
  }

  function hasChildren(item: ContextMenuEntry): item is { label: string; children: ContextMenuEntry[] } {
    return 'children' in item;
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if $contextMenu.visible}
  <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
  <div class="ctx-backdrop" onclick={hideContextMenu} oncontextmenu={(e) => { e.preventDefault(); hideContextMenu(); }}></div>
  <div
    class="ctx-menu"
    bind:this={menuEl}
    style="left: {$contextMenu.x}px; top: {$contextMenu.y}px"
    role="menu"
  >
    {#each $contextMenu.items as item}
      {#if 'separator' in item}
        <hr class="ctx-separator" />
      {:else if hasChildren(item)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="ctx-submenu-trigger"
          onmouseenter={() => openSubmenu = item.label}
          onmouseleave={() => openSubmenu = null}
        >
          <button class="ctx-item" role="menuitem">
            <span class="ctx-label">{item.label}</span>
            <span class="ctx-arrow">&#9656;</span>
          </button>
          {#if openSubmenu === item.label}
            <div class="ctx-submenu" role="menu">
              {#each item.children as child}
                {#if 'separator' in child}
                  <hr class="ctx-separator" />
                {:else if !hasChildren(child)}
                  <button
                    class="ctx-item"
                    class:danger={child.danger}
                    role="menuitem"
                    onclick={() => handleClick(child)}
                  >
                    <span class="ctx-label">{child.label}</span>
                  </button>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <button
          class="ctx-item"
          class:danger={item.danger}
          role="menuitem"
          onclick={() => handleClick(item)}
        >
          <span class="ctx-label">{item.label}</span>
          {#if item.shortcut}
            <span class="ctx-shortcut">{item.shortcut}</span>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 299;
  }
  .ctx-menu {
    position: fixed;
    z-index: 300;
    min-width: 160px;
    max-width: 280px;
    padding: 4px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  }
  .ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 5px 10px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
    text-align: left;
    gap: 16px;
  }
  .ctx-item:hover {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .ctx-item.danger {
    color: var(--danger);
  }
  .ctx-item.danger:hover {
    background: var(--danger);
    color: var(--bg-primary);
  }
  .ctx-label {
    white-space: nowrap;
  }
  .ctx-shortcut {
    font-size: 11px;
    opacity: 0.6;
    white-space: nowrap;
  }
  .ctx-arrow {
    font-size: 10px;
    opacity: 0.6;
  }
  .ctx-separator {
    border: none;
    border-top: 1px solid var(--border);
    margin: 4px 0;
  }
  .ctx-submenu-trigger {
    position: relative;
  }
  .ctx-submenu {
    position: absolute;
    left: 100%;
    top: -4px;
    min-width: 160px;
    max-width: 300px;
    padding: 4px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    z-index: 301;
  }
</style>
