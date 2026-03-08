<script lang="ts">
  interface Props {
    direction?: 'horizontal' | 'vertical';
    initialSize?: number;
    minSize?: number;
    maxSize?: number;
    children: import('svelte').Snippet;
  }

  let { direction = 'vertical', initialSize = 300, minSize = 100, maxSize = 800, children }: Props = $props();

  let size = $state(initialSize);
  let dragging = $state(false);
  let container: HTMLDivElement;

  function onMouseDown(e: MouseEvent) {
    e.preventDefault();
    dragging = true;

    const onMouseMove = (e: MouseEvent) => {
      if (!container) return;
      const rect = container.getBoundingClientRect();
      let newSize: number;
      if (direction === 'horizontal') {
        newSize = e.clientX - rect.left;
      } else {
        newSize = rect.bottom - e.clientY;
      }
      size = Math.max(minSize, Math.min(maxSize, newSize));
    };

    const onMouseUp = () => {
      dragging = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }
</script>

<div
  bind:this={container}
  class="split-pane split-{direction}"
  style={direction === 'horizontal'
    ? `grid-template-columns: ${size}px 4px 1fr`
    : `grid-template-rows: 1fr 4px ${size}px`}
>
  {@render children()}
  <div
    class="divider divider-{direction}"
    class:active={dragging}
    onmousedown={onMouseDown}
    role="separator"
  ></div>
</div>

<style>
  .split-pane {
    display: grid;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  .split-horizontal {
    grid-template-columns: auto 4px 1fr;
  }
  .split-vertical {
    grid-template-rows: 1fr 4px auto;
  }
  .divider {
    background: var(--border);
    z-index: 10;
    transition: background-color 0.15s;
  }
  .divider:hover,
  .divider.active {
    background: var(--accent);
  }
  .divider-horizontal {
    cursor: col-resize;
    grid-row: 1 / -1;
    grid-column: 2;
  }
  .divider-vertical {
    cursor: row-resize;
    grid-column: 1 / -1;
    grid-row: 2;
  }
</style>
