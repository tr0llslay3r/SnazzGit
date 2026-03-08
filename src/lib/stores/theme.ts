import { writable, get } from 'svelte/store';
import type { Theme } from '$lib/types';
import { darkTheme } from '$lib/themes/dark';
import { lollipopTheme } from '$lib/themes/lollipop';
import { neonTheme } from '$lib/themes/neon';
import { classicTheme } from '$lib/themes/classic';

export const builtinThemes: Theme[] = [darkTheme, lollipopTheme, neonTheme, classicTheme];

export const userThemes = writable<Theme[]>([]);
export const activeTheme = writable<Theme>(darkTheme);

export function applyTheme(theme: Theme) {
  activeTheme.set(theme);
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.colors)) {
    root.style.setProperty(`--${key}`, value);
  }
  try {
    localStorage.setItem('snazzgit-theme', theme.name);
  } catch {
    // ignore
  }
}

export function loadSavedTheme() {
  try {
    const savedName = localStorage.getItem('snazzgit-theme');
    if (savedName) {
      const all = [...builtinThemes, ...get(userThemes)];
      const found = all.find((t) => t.name === savedName);
      if (found) {
        applyTheme(found);
        return;
      }
    }
  } catch {
    // ignore
  }
  applyTheme(darkTheme);
}
