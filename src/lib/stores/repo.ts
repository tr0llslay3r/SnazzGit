import { writable, derived, get } from 'svelte/store';
import type { RepoInfo, CommitInfo, GraphRow, WorkingTreeStatus, StashEntry, RecentRepo } from '$lib/types';
import * as tauri from '$lib/utils/tauri';

export const recentRepos = writable<RecentRepo[]>([]);
export const repoInfo = writable<RepoInfo | null>(null);
export const commits = writable<CommitInfo[]>([]);
export const graphRows = writable<GraphRow[]>([]);
export const workingStatus = writable<WorkingTreeStatus | null>(null);
export const stashEntries = writable<StashEntry[]>([]);
export const isLoading = writable(false);
export const repoError = writable<string | null>(null);

export async function refreshRepo() {
  const repo = get(repoInfo);
  if (!repo) return;
  repoInfo.set(await tauri.openRepository(repo.path));
}

export async function refreshCommits() {
  const repo = get(repoInfo);
  if (!repo) return;
  const loaded = await tauri.loadCommits(repo.path, 500, 0);
  commits.set(loaded);
  graphRows.set(await tauri.computeGraph(loaded));
}

export async function refreshStatus() {
  const repo = get(repoInfo);
  if (!repo) return;
  workingStatus.set(await tauri.getStatus(repo.path));
}

export async function refreshStashes() {
  const repo = get(repoInfo);
  if (!repo) return;
  stashEntries.set(await tauri.stashList(repo.path));
}

export async function refreshAll() {
  const repo = get(repoInfo);
  if (!repo) return;
  isLoading.set(true);
  try {
    repoInfo.set(await tauri.openRepository(repo.path));
    await Promise.all([refreshCommits(), refreshStatus(), refreshStashes()]);
  } finally {
    isLoading.set(false);
  }
}

export const localBranches = derived(repoInfo, ($repo) =>
  $repo?.branches.filter((b) => !b.is_remote) ?? []
);

export const remoteBranches = derived(repoInfo, ($repo) =>
  $repo?.branches.filter((b) => b.is_remote) ?? []
);

export const currentBranch = derived(repoInfo, ($repo) => $repo?.current_branch ?? null);

export async function loadRecentRepos() {
  try {
    recentRepos.set(await tauri.loadRecentRepos());
  } catch { /* ignore */ }
}

export function closeRepo() {
  repoInfo.set(null);
  commits.set([]);
  graphRows.set([]);
  workingStatus.set(null);
  stashEntries.set([]);
}

export const hasChanges = derived(workingStatus, ($status) => {
  if (!$status) return false;
  return $status.staged.length > 0 || $status.unstaged.length > 0 || $status.untracked.length > 0;
});
