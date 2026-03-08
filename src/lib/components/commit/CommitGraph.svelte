<script lang="ts">
  import { graphRows } from '$lib/stores/repo';
  import { columnX, rowY, edgePath, graphWidth, nodeRadius, ROW_HEIGHT } from '$lib/utils/graph';

  interface Props {
    scrollTop?: number;
    clientHeight?: number;
    totalRows?: number;
  }

  let { scrollTop = 0, clientHeight = 600, totalRows = 0 }: Props = $props();

  const BUFFER = 10;

  let visibleStart = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER));
  let visibleEnd = $derived(Math.min($graphRows.length, visibleStart + Math.ceil(clientHeight / ROW_HEIGHT) + BUFFER * 2));
  let visibleRows = $derived($graphRows.slice(visibleStart, visibleEnd));

  let maxColumns = $derived(Math.max(...$graphRows.map((r) => r.num_columns), 1));
  let width = $derived(graphWidth(maxColumns));
  let totalHeight = $derived(totalRows * ROW_HEIGHT);

  function getGraphColor(column: number): string {
    return `var(--graph-color-${column % 6})`;
  }
</script>

<div class="commit-graph" style="width: {width}px; height: {totalHeight}px;">
  <svg {width} height={totalHeight} class="graph-svg">
    {#each visibleRows as row, i}
      {@const rowIndex = visibleStart + i}
      {#each row.edges as edge}
        <path
          d={edgePath(edge, rowIndex, rowIndex + 1)}
          stroke={getGraphColor(edge.from_column)}
          stroke-width="2"
          fill="none"
        />
      {/each}
      <circle
        cx={columnX(row.column)}
        cy={rowY(rowIndex)}
        r={nodeRadius()}
        fill={getGraphColor(row.column)}
      />
    {/each}
  </svg>
</div>

<style>
  .commit-graph {
    flex-shrink: 0;
    position: relative;
  }
  .graph-svg {
    display: block;
    position: absolute;
    top: 0;
    left: 0;
  }
</style>
