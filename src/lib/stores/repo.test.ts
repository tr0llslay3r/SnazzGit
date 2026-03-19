import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('$lib/utils/tauri');

import * as tauri from '$lib/utils/tauri';
import { listen } from '@tauri-apps/api/event';
import type { RepoInfo, BranchInfo, WorkingTreeStatus, StashEntry } from '$lib/types';
import {
  repoInfo,
  commits,
  graphRows,
  workingStatus,
  stashEntries,
  isLoading,
  repoError,
  recentRepos,
  localBranches,
  remoteBranches,
  currentBranch,
  hasChanges,
  refreshRepo,
  refreshCommits,
  refreshStatus,
  refreshStashes,
  refreshAll,
  loadRecentRepos,
  closeRepo,
  setupWatcher,
  teardownWatcher,
  startAutoFetch,
  stopAutoFetch,
} from '$lib/stores/repo';

function makeBranch(overrides?: Partial<BranchInfo>): BranchInfo {
  return {
    name: 'main',
    is_head: true,
    is_remote: false,
    upstream: null,
    commit_id: 'abc123',
    ahead: null,
    behind: null,
    ...overrides,
  };
}

function makeRepoInfo(overrides?: Partial<RepoInfo>): RepoInfo {
  return {
    path: '/test/repo',
    name: 'test-repo',
    current_branch: 'main',
    is_bare: false,
    branches: [],
    remotes: [],
    tags: [],
    stash_count: 0,
    ...overrides,
  };
}

function makeStatus(overrides?: Partial<WorkingTreeStatus>): WorkingTreeStatus {
  return {
    staged: [],
    unstaged: [],
    untracked: [],
    conflicted: [],
    ...overrides,
  };
}

beforeEach(() => {
  repoInfo.set(null);
  commits.set([]);
  graphRows.set([]);
  workingStatus.set(null);
  stashEntries.set([]);
  isLoading.set(false);
  repoError.set(null);
  recentRepos.set([]);
  stopAutoFetch();
  vi.clearAllMocks();
});

describe('refreshRepo', () => {
  it('does nothing when repoInfo is null', async () => {
    await refreshRepo();
    expect(tauri.openRepository).not.toHaveBeenCalled();
  });

  it('calls openRepository and updates store', async () => {
    const initial = makeRepoInfo();
    const updated = makeRepoInfo({ current_branch: 'develop' });
    repoInfo.set(initial);
    vi.mocked(tauri.openRepository).mockResolvedValue(updated);

    await refreshRepo();

    expect(tauri.openRepository).toHaveBeenCalledWith('/test/repo');
    expect(get(repoInfo)).toEqual(updated);
  });
});

describe('refreshCommits', () => {
  it('does nothing when repoInfo is null', async () => {
    await refreshCommits();
    expect(tauri.loadCommits).not.toHaveBeenCalled();
  });

  it('loads commits and computes graph', async () => {
    repoInfo.set(makeRepoInfo());
    const mockCommits = [{ id: '1', short_id: '1' }] as any;
    const mockGraph = [{ columns: [] }] as any;
    vi.mocked(tauri.loadCommits).mockResolvedValue(mockCommits);
    vi.mocked(tauri.computeGraph).mockResolvedValue(mockGraph);

    await refreshCommits();

    expect(tauri.loadCommits).toHaveBeenCalledWith('/test/repo', 500, 0);
    expect(tauri.computeGraph).toHaveBeenCalledWith(mockCommits);
    expect(get(commits)).toEqual(mockCommits);
    expect(get(graphRows)).toEqual(mockGraph);
  });
});

describe('refreshStatus', () => {
  it('does nothing when repoInfo is null', async () => {
    await refreshStatus();
    expect(tauri.getStatus).not.toHaveBeenCalled();
  });

  it('calls getStatus and updates workingStatus store', async () => {
    repoInfo.set(makeRepoInfo());
    const status = makeStatus({ untracked: [{ path: 'new.txt', status: 'new' } as any] });
    vi.mocked(tauri.getStatus).mockResolvedValue(status);

    await refreshStatus();

    expect(tauri.getStatus).toHaveBeenCalledWith('/test/repo');
    expect(get(workingStatus)).toEqual(status);
  });
});

describe('refreshStashes', () => {
  it('does nothing when repoInfo is null', async () => {
    await refreshStashes();
    expect(tauri.stashList).not.toHaveBeenCalled();
  });

  it('calls stashList and updates stashEntries store', async () => {
    repoInfo.set(makeRepoInfo());
    const stashes: StashEntry[] = [{ index: 0, message: 'WIP', commit_id: 'abc' }];
    vi.mocked(tauri.stashList).mockResolvedValue(stashes);

    await refreshStashes();

    expect(tauri.stashList).toHaveBeenCalledWith('/test/repo');
    expect(get(stashEntries)).toEqual(stashes);
  });
});

describe('refreshAll', () => {
  it('does nothing when repoInfo is null', async () => {
    await refreshAll();
    expect(tauri.openRepository).not.toHaveBeenCalled();
    expect(get(isLoading)).toBe(false);
  });

  it('sets isLoading true during refresh and false after', async () => {
    repoInfo.set(makeRepoInfo());
    const loadingStates: boolean[] = [];

    vi.mocked(tauri.openRepository).mockImplementation(async () => {
      loadingStates.push(get(isLoading));
      return makeRepoInfo();
    });
    vi.mocked(tauri.loadCommits).mockResolvedValue([]);
    vi.mocked(tauri.computeGraph).mockResolvedValue([]);
    vi.mocked(tauri.getStatus).mockResolvedValue(makeStatus());
    vi.mocked(tauri.stashList).mockResolvedValue([]);

    await refreshAll();

    expect(loadingStates[0]).toBe(true);
    expect(get(isLoading)).toBe(false);
  });

  it('sets isLoading false even when an error occurs', async () => {
    repoInfo.set(makeRepoInfo());
    vi.mocked(tauri.openRepository).mockRejectedValue(new Error('fail'));

    await refreshAll().catch(() => {});

    expect(get(isLoading)).toBe(false);
  });

  it('calls all refresh sub-functions', async () => {
    repoInfo.set(makeRepoInfo());
    vi.mocked(tauri.openRepository).mockResolvedValue(makeRepoInfo());
    vi.mocked(tauri.loadCommits).mockResolvedValue([]);
    vi.mocked(tauri.computeGraph).mockResolvedValue([]);
    vi.mocked(tauri.getStatus).mockResolvedValue(makeStatus());
    vi.mocked(tauri.stashList).mockResolvedValue([]);

    await refreshAll();

    expect(tauri.openRepository).toHaveBeenCalled();
    expect(tauri.loadCommits).toHaveBeenCalled();
    expect(tauri.getStatus).toHaveBeenCalled();
    expect(tauri.stashList).toHaveBeenCalled();
  });
});

describe('derived stores', () => {
  it('localBranches returns empty array when repoInfo is null', () => {
    expect(get(localBranches)).toEqual([]);
  });

  it('localBranches filters out remote branches', () => {
    repoInfo.set(
      makeRepoInfo({
        branches: [
          makeBranch({ name: 'main', is_remote: false }),
          makeBranch({ name: 'origin/main', is_remote: true }),
          makeBranch({ name: 'develop', is_remote: false }),
        ],
      }),
    );
    const result = get(localBranches);
    expect(result).toHaveLength(2);
    expect(result.map((b) => b.name)).toEqual(['main', 'develop']);
  });

  it('remoteBranches returns only remote branches', () => {
    repoInfo.set(
      makeRepoInfo({
        branches: [
          makeBranch({ name: 'main', is_remote: false }),
          makeBranch({ name: 'origin/main', is_remote: true }),
        ],
      }),
    );
    const result = get(remoteBranches);
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('origin/main');
  });

  it('remoteBranches returns empty array when repoInfo is null', () => {
    expect(get(remoteBranches)).toEqual([]);
  });

  it('currentBranch returns current_branch from repoInfo', () => {
    repoInfo.set(makeRepoInfo({ current_branch: 'feature/x' }));
    expect(get(currentBranch)).toBe('feature/x');
  });

  it('currentBranch returns null when repoInfo is null', () => {
    expect(get(currentBranch)).toBeNull();
  });

  it('hasChanges returns false when workingStatus is null', () => {
    expect(get(hasChanges)).toBe(false);
  });

  it('hasChanges returns false when all arrays are empty', () => {
    workingStatus.set(makeStatus());
    expect(get(hasChanges)).toBe(false);
  });

  it('hasChanges returns true when staged files exist', () => {
    workingStatus.set(makeStatus({ staged: [{ path: 'a.ts', status: 'modified' } as any] }));
    expect(get(hasChanges)).toBe(true);
  });

  it('hasChanges returns true when unstaged files exist', () => {
    workingStatus.set(makeStatus({ unstaged: [{ path: 'b.ts', status: 'modified' } as any] }));
    expect(get(hasChanges)).toBe(true);
  });

  it('hasChanges returns true when untracked files exist', () => {
    workingStatus.set(makeStatus({ untracked: [{ path: 'c.ts', status: 'new' } as any] }));
    expect(get(hasChanges)).toBe(true);
  });
});

describe('loadRecentRepos', () => {
  it('sets recentRepos store on success', async () => {
    const repos = [{ path: '/a', name: 'a', last_opened: 0 }];
    vi.mocked(tauri.loadRecentRepos).mockResolvedValue(repos);

    await loadRecentRepos();

    expect(get(recentRepos)).toEqual(repos);
  });

  it('silently ignores errors', async () => {
    vi.mocked(tauri.loadRecentRepos).mockRejectedValue(new Error('fail'));

    await loadRecentRepos();

    expect(get(recentRepos)).toEqual([]);
  });
});

describe('closeRepo', () => {
  it('resets all stores to initial values', () => {
    repoInfo.set(makeRepoInfo());
    commits.set([{ id: '1' }] as any);
    graphRows.set([{ columns: [] }] as any);
    workingStatus.set(makeStatus());
    stashEntries.set([{ index: 0, message: 'WIP', commit_id: 'x' }]);

    closeRepo();

    expect(get(repoInfo)).toBeNull();
    expect(get(commits)).toEqual([]);
    expect(get(graphRows)).toEqual([]);
    expect(get(workingStatus)).toBeNull();
    expect(get(stashEntries)).toEqual([]);
  });
});

describe('setupWatcher', () => {
  it('calls startWatching and listen', async () => {
    const unlisten = vi.fn();
    vi.mocked(tauri.startWatching).mockResolvedValue(undefined);
    vi.mocked(listen).mockResolvedValue(unlisten);

    await setupWatcher('/test/repo');

    expect(tauri.startWatching).toHaveBeenCalledWith('/test/repo');
    expect(vi.mocked(listen)).toHaveBeenCalledWith('fs-changed', expect.any(Function));
  });

  it('fs-changed callback triggers refreshStatus', async () => {
    const unlisten = vi.fn();
    vi.mocked(tauri.startWatching).mockResolvedValue(undefined);
    vi.mocked(listen).mockResolvedValue(unlisten);
    repoInfo.set(makeRepoInfo());
    vi.mocked(tauri.getStatus).mockResolvedValue(makeStatus());

    await setupWatcher('/test/repo');

    // Get the callback passed to listen and invoke it
    const callback = vi.mocked(listen).mock.calls[0][1];
    await callback({});

    expect(tauri.getStatus).toHaveBeenCalledWith('/test/repo');
  });

  it('still starts auto-fetch even if startWatching fails', async () => {
    vi.mocked(tauri.startWatching).mockRejectedValue(new Error('fail'));
    vi.useFakeTimers();

    await setupWatcher('/test/repo');

    // Auto-fetch should have been started — verify interval was created
    // by checking that advancing time triggers fetchRemote (when repo has remotes)
    repoInfo.set(makeRepoInfo({ remotes: ['origin'] }));
    vi.mocked(tauri.fetchRemote).mockResolvedValue(undefined);
    vi.mocked(tauri.openRepository).mockResolvedValue(makeRepoInfo({ remotes: ['origin'] }));

    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    expect(tauri.fetchRemote).toHaveBeenCalled();

    vi.useRealTimers();
  });
});

describe('teardownWatcher', () => {
  it('calls unlisten and stopWatching', async () => {
    const unlisten = vi.fn();
    vi.mocked(tauri.startWatching).mockResolvedValue(undefined);
    vi.mocked(listen).mockResolvedValue(unlisten);

    await setupWatcher('/test/repo');
    await teardownWatcher();

    expect(unlisten).toHaveBeenCalled();
    expect(tauri.stopWatching).toHaveBeenCalled();
  });

  it('handles stopWatching errors silently', async () => {
    vi.mocked(tauri.stopWatching).mockRejectedValue(new Error('fail'));

    await teardownWatcher(); // should not throw
  });
});

describe('auto-fetch', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    stopAutoFetch();
    vi.useRealTimers();
  });

  it('fetches first remote after 5 minutes', async () => {
    repoInfo.set(makeRepoInfo({ remotes: ['origin'] }));
    vi.mocked(tauri.fetchRemote).mockResolvedValue(undefined);
    vi.mocked(tauri.openRepository).mockResolvedValue(makeRepoInfo({ remotes: ['origin'] }));

    startAutoFetch();
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    expect(tauri.fetchRemote).toHaveBeenCalledWith('/test/repo', 'origin');
    expect(tauri.openRepository).toHaveBeenCalledWith('/test/repo');
  });

  it('skips fetch when repoInfo is null', async () => {
    startAutoFetch();
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    expect(tauri.fetchRemote).not.toHaveBeenCalled();
  });

  it('skips fetch when repo has no remotes', async () => {
    repoInfo.set(makeRepoInfo({ remotes: [] }));

    startAutoFetch();
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    expect(tauri.fetchRemote).not.toHaveBeenCalled();
  });

  it('silently ignores fetch errors', async () => {
    repoInfo.set(makeRepoInfo({ remotes: ['origin'] }));
    vi.mocked(tauri.fetchRemote).mockRejectedValue(new Error('network'));

    startAutoFetch();
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    // Should not throw and should not crash the interval
    expect(tauri.fetchRemote).toHaveBeenCalled();
  });

  it('stopAutoFetch prevents further fetches', async () => {
    repoInfo.set(makeRepoInfo({ remotes: ['origin'] }));
    vi.mocked(tauri.fetchRemote).mockResolvedValue(undefined);
    vi.mocked(tauri.openRepository).mockResolvedValue(makeRepoInfo({ remotes: ['origin'] }));

    startAutoFetch();
    stopAutoFetch();
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    expect(tauri.fetchRemote).not.toHaveBeenCalled();
  });

  it('calling startAutoFetch twice does not double-fetch', async () => {
    repoInfo.set(makeRepoInfo({ remotes: ['origin'] }));
    vi.mocked(tauri.fetchRemote).mockResolvedValue(undefined);
    vi.mocked(tauri.openRepository).mockResolvedValue(makeRepoInfo({ remotes: ['origin'] }));

    startAutoFetch();
    startAutoFetch(); // should clear the first interval
    await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

    // Should only be called once, not twice
    expect(tauri.fetchRemote).toHaveBeenCalledTimes(1);
  });
});
