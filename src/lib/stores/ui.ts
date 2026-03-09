import { writable } from 'svelte/store';
import type { CommitInfo, Toast } from '$lib/types';

export const selectedCommit = writable<CommitInfo | null>(null);
export const selectedFile = writable<string | null>(null);
export const selectedFileStaged = writable(false);
export const showStagingArea = writable(true);
export const diffMode = writable<'unified' | 'split'>('unified');
export const sidebarWidth = writable(240);
export const bottomPaneHeight = writable(300);
export const showSearch = writable(false);
export const searchQuery = writable('');
export const toasts = writable<Toast[]>([]);
export const showThemePicker = writable(false);
export const showBranchDialog = writable(false);
export const showStashDialog = writable(false);
export const showMergeDialog = writable(false);
export const jumpToCommitId = writable<string | null>(null);

let toastId = 0;

export function addToast(message: string, type: Toast['type'] = 'info', timeout = 4000) {
  const id = String(++toastId);
  toasts.update((t) => [...t, { id, message, type, timeout }]);
  if (timeout > 0) {
    setTimeout(() => {
      toasts.update((t) => t.filter((toast) => toast.id !== id));
    }, timeout);
  }
}

export function removeToast(id: string) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
