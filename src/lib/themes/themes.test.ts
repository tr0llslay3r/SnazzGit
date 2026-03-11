import { describe, it, expect } from 'vitest';
import { darkTheme } from '$lib/themes/dark';
import { lollipopTheme } from '$lib/themes/lollipop';
import { neonTheme } from '$lib/themes/neon';
import { classicTheme } from '$lib/themes/classic';
import type { Theme } from '$lib/types';

const ALL_THEMES: Theme[] = [darkTheme, lollipopTheme, neonTheme, classicTheme];

const REQUIRED_COLOR_KEYS = [
  'bg-primary',
  'bg-secondary',
  'bg-surface',
  'text-primary',
  'text-secondary',
  'accent',
  'danger',
  'success',
  'warning',
  'border',
  'diff-added-bg',
  'diff-removed-bg',
  'diff-added-text',
  'diff-removed-text',
  'graph-color-0',
  'graph-color-1',
  'graph-color-2',
  'graph-color-3',
  'graph-color-4',
  'graph-color-5',
];

describe('built-in themes', () => {
  it('there are exactly 4 built-in themes', () => {
    expect(ALL_THEMES).toHaveLength(4);
  });

  it('each theme has a non-empty name', () => {
    for (const theme of ALL_THEMES) {
      expect(theme.name).toBeTruthy();
    }
  });

  it('all theme names are unique', () => {
    const names = ALL_THEMES.map((t) => t.name);
    const unique = new Set(names);
    expect(unique.size).toBe(names.length);
  });

  it('each theme has a colors object', () => {
    for (const theme of ALL_THEMES) {
      expect(typeof theme.colors).toBe('object');
      expect(theme.colors).not.toBeNull();
    }
  });

  for (const key of REQUIRED_COLOR_KEYS) {
    it(`every theme defines color key "${key}"`, () => {
      for (const theme of ALL_THEMES) {
        expect(theme.colors).toHaveProperty(key);
        expect(theme.colors[key]).toBeTruthy();
      }
    });
  }

  it('all color values look like CSS color strings', () => {
    for (const theme of ALL_THEMES) {
      for (const [key, value] of Object.entries(theme.colors)) {
        expect(
          value.startsWith('#') || value.startsWith('rgb') || value.startsWith('hsl'),
          `${theme.name}.${key} = "${value}" is not a recognisable CSS color`,
        ).toBe(true);
      }
    }
  });
});

describe('darkTheme', () => {
  it('has name "Dark"', () => {
    expect(darkTheme.name).toBe('Dark');
  });
});

describe('lollipopTheme', () => {
  it('has a non-empty name', () => {
    expect(lollipopTheme.name).toBeTruthy();
  });
});

describe('neonTheme', () => {
  it('has a non-empty name', () => {
    expect(neonTheme.name).toBeTruthy();
  });
});

describe('classicTheme', () => {
  it('has a non-empty name', () => {
    expect(classicTheme.name).toBeTruthy();
  });
});
