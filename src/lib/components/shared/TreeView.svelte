<script lang="ts">
  import TreeView from './TreeView.svelte';

  interface TreeNode {
    label: string;
    icon?: string;
    children?: TreeNode[];
    data?: unknown;
    badge?: string | number;
  }

  interface Props {
    nodes: TreeNode[];
    onSelect?: (node: TreeNode) => void;
    onContextMenu?: (node: TreeNode, event: MouseEvent) => void;
    depth?: number;
  }

  let { nodes, onSelect, onContextMenu, depth = 0 }: Props = $props();

  let expanded: Record<string, boolean> = $state({});

  function toggle(label: string) {
    expanded[label] = !expanded[label];
  }
</script>

{#each nodes as node}
  <div class="tree-node" style="padding-left: {depth * 16 + 8}px">
    {#if node.children && node.children.length > 0}
      <button
        class="tree-toggle"
        onclick={() => toggle(node.label)}
        aria-label="Toggle {node.label}"
      >
        <svg class="chevron" class:open={expanded[node.label]} viewBox="0 0 16 16" width="12" height="12">
          <path d="M6 4l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
      </button>
    {:else}
      <span class="tree-spacer"></span>
    {/if}
    <button
      class="tree-label"
      onclick={() => onSelect?.(node)}
      oncontextmenu={(e) => { if (onContextMenu) { e.preventDefault(); e.stopPropagation(); onContextMenu(node, e); } }}
    >
      {#if node.icon}
        <span class="tree-icon">{node.icon}</span>
      {/if}
      <span class="tree-text">{node.label}</span>
      {#if node.badge !== undefined}
        <span class="tree-badge">{node.badge}</span>
      {/if}
    </button>
  </div>
  {#if node.children && expanded[node.label]}
    <TreeView nodes={node.children} {onSelect} {onContextMenu} depth={depth + 1} />
  {/if}
{/each}

<style>
  .tree-node {
    display: flex;
    align-items: center;
    height: 26px;
    gap: 2px;
  }
  .tree-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }
  .tree-toggle:hover {
    color: var(--text-primary);
  }
  .chevron {
    transition: transform 0.15s;
  }
  .chevron.open {
    transform: rotate(90deg);
  }
  .tree-spacer {
    width: 18px;
    flex-shrink: 0;
  }
  .tree-label {
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    text-align: left;
  }
  .tree-label:hover {
    background: var(--bg-surface);
  }
  .tree-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
  .tree-text {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tree-badge {
    font-size: 11px;
    padding: 0 6px;
    border-radius: 10px;
    background: var(--accent);
    color: var(--bg-primary);
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
