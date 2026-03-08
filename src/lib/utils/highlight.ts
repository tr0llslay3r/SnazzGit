import type { HighlightSpan } from '$lib/types';

export function applyHighlightSpans(content: string, spans: HighlightSpan[]): string {
  if (!spans.length) return escapeHtml(content);

  let result = '';
  let lastIndex = 0;

  for (const span of spans) {
    if (span.start > lastIndex) {
      result += escapeHtml(content.slice(lastIndex, span.start));
    }
    result += `<span style="${escapeAttr(span.style)}">${escapeHtml(content.slice(span.start, span.end))}</span>`;
    lastIndex = span.end;
  }

  if (lastIndex < content.length) {
    result += escapeHtml(content.slice(lastIndex));
  }

  return result;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function escapeAttr(text: string): string {
  return text.replace(/"/g, '&quot;');
}
