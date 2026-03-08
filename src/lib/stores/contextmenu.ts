import { writable } from 'svelte/store';

export type ContextMenuEntry =
  | { label: string; action: () => void; shortcut?: string; danger?: boolean }
  | { label: string; children: ContextMenuEntry[] }
  | { separator: true };

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  items: ContextMenuEntry[];
}

export const contextMenu = writable<ContextMenuState>({
  visible: false,
  x: 0,
  y: 0,
  items: [],
});

export function showContextMenu(x: number, y: number, items: ContextMenuEntry[]) {
  contextMenu.set({ visible: true, x, y, items });
}

export function hideContextMenu() {
  contextMenu.update((s) => ({ ...s, visible: false }));
}
