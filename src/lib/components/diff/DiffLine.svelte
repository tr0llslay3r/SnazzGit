<script lang="ts">
  import type { DiffLine } from '$lib/types';
  import { applyHighlightSpans } from '$lib/utils/highlight';

  interface Props {
    line: DiffLine;
  }

  let { line }: Props = $props();

  let bgClass = $derived(
    line.line_type === 'Addition' ? 'line-added' :
    line.line_type === 'Deletion' ? 'line-removed' : 'line-context'
  );

  let prefix = $derived(
    line.line_type === 'Addition' ? '+' :
    line.line_type === 'Deletion' ? '-' : ' '
  );

  let highlighted = $derived(applyHighlightSpans(line.content, line.spans));
</script>

<div class="diff-line {bgClass}">
  <span class="line-no old">{line.old_lineno ?? ''}</span>
  <span class="line-no new">{line.new_lineno ?? ''}</span>
  <span class="line-prefix">{prefix}</span>
  <span class="line-content">{@html highlighted}</span>
</div>

<style>
  .diff-line {
    display: flex;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 12px;
    line-height: 20px;
    white-space: pre;
  }
  .line-added {
    background: var(--diff-added-bg);
    color: var(--diff-added-text);
  }
  .line-removed {
    background: var(--diff-removed-bg);
    color: var(--diff-removed-text);
  }
  .line-context {
    color: var(--text-primary);
  }
  .line-no {
    width: 45px;
    text-align: right;
    padding-right: 8px;
    color: var(--text-secondary);
    opacity: 0.6;
    user-select: none;
    flex-shrink: 0;
  }
  .line-prefix {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    user-select: none;
  }
  .line-content {
    flex: 1;
    padding-right: 12px;
  }
</style>
