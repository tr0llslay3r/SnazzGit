import { invoke } from '@tauri-apps/api/core';
import type {
  RepoInfo,
  CommitInfo,
  GraphRow,
  WorkingTreeStatus,
  DiffFile,
  StashEntry,
  BlameInfo,
  Theme,
  RecentRepo,
} from '$lib/types';

// CLI
export const getCliArgs = () =>
  invoke<string[]>('get_cli_args');

// Repository
export const openRepository = (path: string) =>
  invoke<RepoInfo>('open_repository', { path });

export const initRepository = (path: string) =>
  invoke<RepoInfo>('init_repository', { path });

// Commits
export const loadCommits = (path: string, limit: number, offset: number) =>
  invoke<CommitInfo[]>('load_commits', { path, limit, offset });

export const getCommitDetail = (path: string, commitId: string) =>
  invoke<CommitInfo>('get_commit_detail', { path, commitId });

export const computeGraph = (commits: CommitInfo[]) =>
  invoke<GraphRow[]>('compute_graph', { commits });

export const createCommit = (path: string, message: string, amend: boolean) =>
  invoke<string>('create_commit', { path, message, amend });

// Branches
export const createBranch = (path: string, name: string) =>
  invoke<void>('create_branch', { path, name });

export const checkoutBranch = (path: string, name: string) =>
  invoke<void>('checkout_branch', { path, name });

export const deleteBranch = (path: string, name: string) =>
  invoke<void>('delete_branch', { path, name });

export const renameBranch = (path: string, oldName: string, newName: string) =>
  invoke<void>('rename_branch', { path, oldName, newName });

export const mergeBranch = (path: string, sourceBranch: string) =>
  invoke<string>('merge_branch', { path, sourceBranch });

export const resetToCommit = (path: string, commitId: string, mode: string) =>
  invoke<void>('reset_to_commit', { path, commitId, mode });

// Remotes
export const fetchRemote = (path: string, remoteName: string) =>
  invoke<void>('fetch_remote', { path, remoteName });

export const pull = (path: string, remoteName: string) =>
  invoke<string>('pull', { path, remoteName });

export const push = (path: string, remoteName: string) =>
  invoke<void>('push', { path, remoteName });

// Diff
export const getWorkingDiff = (path: string, filePath: string, staged: boolean) =>
  invoke<DiffFile>('get_working_diff', { path, filePath, staged });

export const getCommitDiff = (path: string, commitId: string, filePath?: string) =>
  invoke<DiffFile[]>('get_commit_diff', { path, commitId, filePath });

// Status
export const getStatus = (path: string) =>
  invoke<WorkingTreeStatus>('get_status', { path });

export const stageFile = (path: string, filePath: string) =>
  invoke<void>('stage_file', { path, filePath });

export const unstageFile = (path: string, filePath: string) =>
  invoke<void>('unstage_file', { path, filePath });

export const stageAll = (path: string) =>
  invoke<void>('stage_all', { path });

export const unstageAll = (path: string) =>
  invoke<void>('unstage_all', { path });

export const discardFile = (path: string, filePath: string) =>
  invoke<void>('discard_file', { path, filePath });

export const deleteFile = (path: string, filePath: string) =>
  invoke<void>('delete_file', { path, filePath });

export const addToGitignore = (path: string, pattern: string) =>
  invoke<void>('add_to_gitignore', { path, pattern });

// Stash
export const stashList = (path: string) =>
  invoke<StashEntry[]>('stash_list', { path });

export const stashSave = (path: string, message: string) =>
  invoke<void>('stash_save', { path, message });

export const stashPop = (path: string, index: number) =>
  invoke<void>('stash_pop', { path, index });

export const stashApply = (path: string, index: number) =>
  invoke<void>('stash_apply', { path, index });

export const stashDrop = (path: string, index: number) =>
  invoke<void>('stash_drop', { path, index });

// Blame
export const getBlame = (path: string, filePath: string) =>
  invoke<BlameInfo>('get_blame', { path, filePath });

// Search
export const searchCommits = (path: string, query: string, maxResults: number) =>
  invoke<CommitInfo[]>('search_commits', { path, query, maxResults });

// Theme
export const loadUserThemes = () =>
  invoke<Theme[]>('load_user_themes');

export const saveUserTheme = (themeData: Theme) =>
  invoke<void>('save_user_theme', { themeData });

export const deleteUserTheme = (name: string) =>
  invoke<void>('delete_user_theme', { name });

// Recent repos
export const loadRecentRepos = () =>
  invoke<RecentRepo[]>('load_recent_repos');

export const addRecentRepo = (path: string, name: string) =>
  invoke<RecentRepo[]>('add_recent_repo', { path, name });

export const removeRecentRepo = (path: string) =>
  invoke<RecentRepo[]>('remove_recent_repo', { path });
