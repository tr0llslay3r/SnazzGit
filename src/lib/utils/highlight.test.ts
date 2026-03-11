import { describe, it, expect } from 'vitest';
import { applyHighlightSpans } from '$lib/utils/highlight';
import type { HighlightSpan } from '$lib/types';

describe('applyHighlightSpans', () => {
  it('returns plain escaped text when spans array is empty', () => {
    expect(applyHighlightSpans('hello world', [])).toBe('hello world');
  });

  it('escapes < and > characters', () => {
    expect(applyHighlightSpans('<b>hi</b>', [])).toBe('&lt;b&gt;hi&lt;/b&gt;');
  });

  it('escapes & characters', () => {
    expect(applyHighlightSpans('a & b', [])).toBe('a &amp; b');
  });

  it('escapes all three html entities in one string', () => {
    const result = applyHighlightSpans('a < b & c > d', []);
    expect(result).toBe('a &lt; b &amp; c &gt; d');
  });

  it('wraps a span in a <span> tag with the given style', () => {
    const spans: HighlightSpan[] = [{ start: 0, end: 5, style: 'color: red' }];
    expect(applyHighlightSpans('hello world', spans)).toBe(
      '<span style="color: red">hello</span> world',
    );
  });

  it('handles a span at the end of the content', () => {
    const spans: HighlightSpan[] = [{ start: 6, end: 11, style: 'color: blue' }];
    expect(applyHighlightSpans('hello world', spans)).toBe(
      'hello <span style="color: blue">world</span>',
    );
  });

  it('handles a span that covers the entire content', () => {
    const spans: HighlightSpan[] = [{ start: 0, end: 5, style: 'font-weight: bold' }];
    expect(applyHighlightSpans('hello', spans)).toBe(
      '<span style="font-weight: bold">hello</span>',
    );
  });

  it('handles multiple non-overlapping spans', () => {
    const spans: HighlightSpan[] = [
      { start: 0, end: 3, style: 'color: red' },
      { start: 4, end: 7, style: 'color: blue' },
    ];
    const result = applyHighlightSpans('foo bar baz', spans);
    expect(result).toContain('<span style="color: red">foo</span>');
    expect(result).toContain('<span style="color: blue">bar</span>');
    expect(result).toContain(' baz');
  });

  it('escapes double-quotes in span style attribute', () => {
    const spans: HighlightSpan[] = [{ start: 0, end: 3, style: 'font-family: "serif"' }];
    const result = applyHighlightSpans('foo', spans);
    expect(result).toContain('style="font-family: &quot;serif&quot;"');
  });

  it('preserves text between spans', () => {
    // spans cover "ab" (0..2) and "ef" (6..8); text between them is "_cd_"
    const spans: HighlightSpan[] = [
      { start: 0, end: 2, style: 'a' },
      { start: 6, end: 8, style: 'b' },
    ];
    const result = applyHighlightSpans('ab_cd_ef', spans);
    expect(result).toContain('_cd_');
  });

  it('escapes html inside a span content', () => {
    const spans: HighlightSpan[] = [{ start: 0, end: 5, style: 'color: green' }];
    const result = applyHighlightSpans('<div>', spans);
    expect(result).toContain('&lt;div&gt;');
    expect(result).not.toContain('<div>');
  });
});
