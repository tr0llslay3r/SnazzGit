import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
  getCliArgs,
  loadUserThemes,
  stopWatching,
  openRepository,
  stageAll,
  rebaseAbort,
  loadCommits,
  checkoutBranch,
  getWorkingDiff,
  createBranch,
  createTag,
  readFileAtRef,
  setUpstream,
  stashSave,
  fetchRemote,
  stageHunk,
  computeGraph,
  createCommit,
  stashList,
  stashDiff,
  searchCommits,
  openInFileManager,
  openTerminal,
  forceDeleteBranch,
  addRemote,
} from '$lib/utils/tauri';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('no-arg commands', () => {
  it('getCliArgs invokes get_cli_args', async () => {
    await getCliArgs();
    expect(invoke).toHaveBeenCalledWith('get_cli_args');
  });

  it('loadUserThemes invokes load_user_themes', async () => {
    await loadUserThemes();
    expect(invoke).toHaveBeenCalledWith('load_user_themes');
  });

  it('stopWatching invokes stop_watching', async () => {
    await stopWatching();
    expect(invoke).toHaveBeenCalledWith('stop_watching');
  });
});

describe('single-arg commands', () => {
  it('openRepository invokes open_repository with path', async () => {
    await openRepository('/repo');
    expect(invoke).toHaveBeenCalledWith('open_repository', { path: '/repo' });
  });

  it('stageAll invokes stage_all with path', async () => {
    await stageAll('/repo');
    expect(invoke).toHaveBeenCalledWith('stage_all', { path: '/repo' });
  });

  it('rebaseAbort invokes rebase_abort with path', async () => {
    await rebaseAbort('/repo');
    expect(invoke).toHaveBeenCalledWith('rebase_abort', { path: '/repo' });
  });
});

describe('multi-arg commands', () => {
  it('loadCommits invokes load_commits with path, limit, offset', async () => {
    await loadCommits('/repo', 100, 0);
    expect(invoke).toHaveBeenCalledWith('load_commits', { path: '/repo', limit: 100, offset: 0, includeRemote: false });
  });

  it('loadCommits passes includeRemote when true', async () => {
    await loadCommits('/repo', 100, 0, true);
    expect(invoke).toHaveBeenCalledWith('load_commits', { path: '/repo', limit: 100, offset: 0, includeRemote: true });
  });

  it('checkoutBranch invokes checkout_branch with path and name', async () => {
    await checkoutBranch('/repo', 'main');
    expect(invoke).toHaveBeenCalledWith('checkout_branch', { path: '/repo', name: 'main' });
  });

  it('getWorkingDiff invokes get_working_diff with path, filePath, staged', async () => {
    await getWorkingDiff('/repo', 'src/main.ts', true);
    expect(invoke).toHaveBeenCalledWith('get_working_diff', {
      path: '/repo',
      filePath: 'src/main.ts',
      staged: true,
    });
  });

  it('createCommit invokes create_commit with path, message, amend', async () => {
    await createCommit('/repo', 'fix: bug', false);
    expect(invoke).toHaveBeenCalledWith('create_commit', {
      path: '/repo',
      message: 'fix: bug',
      amend: false,
    });
  });

  it('computeGraph invokes compute_graph with commits array', async () => {
    const commits = [{ id: 'abc' }] as any;
    await computeGraph(commits);
    expect(invoke).toHaveBeenCalledWith('compute_graph', { commits });
  });

  it('searchCommits invokes search_commits with path, query, maxResults', async () => {
    await searchCommits('/repo', 'fix', 50);
    expect(invoke).toHaveBeenCalledWith('search_commits', {
      path: '/repo',
      query: 'fix',
      maxResults: 50,
    });
  });

  it('stashList invokes stash_list with path', async () => {
    await stashList('/repo');
    expect(invoke).toHaveBeenCalledWith('stash_list', { path: '/repo' });
  });

  it('stashDiff invokes stash_diff with path and stashCommitId', async () => {
    await stashDiff('/repo', 'abc123');
    expect(invoke).toHaveBeenCalledWith('stash_diff', { path: '/repo', stashCommitId: 'abc123' });
  });

  it('forceDeleteBranch invokes force_delete_branch', async () => {
    await forceDeleteBranch('/repo', 'old-branch');
    expect(invoke).toHaveBeenCalledWith('force_delete_branch', { path: '/repo', name: 'old-branch' });
  });

  it('addRemote invokes add_remote with path, name, url', async () => {
    await addRemote('/repo', 'upstream', 'https://example.com/repo.git');
    expect(invoke).toHaveBeenCalledWith('add_remote', {
      path: '/repo',
      name: 'upstream',
      url: 'https://example.com/repo.git',
    });
  });

  it('openInFileManager invokes open_in_file_manager', async () => {
    await openInFileManager('/repo');
    expect(invoke).toHaveBeenCalledWith('open_in_file_manager', { path: '/repo' });
  });

  it('openTerminal invokes open_terminal', async () => {
    await openTerminal('/repo');
    expect(invoke).toHaveBeenCalledWith('open_terminal', { path: '/repo' });
  });
});

describe('optional arg commands (with ?? null)', () => {
  it('createBranch passes commitId when provided', async () => {
    await createBranch('/repo', 'feat', 'abc123');
    expect(invoke).toHaveBeenCalledWith('create_branch', {
      path: '/repo',
      name: 'feat',
      commitId: 'abc123',
    });
  });

  it('createBranch passes null when commitId omitted', async () => {
    await createBranch('/repo', 'feat');
    expect(invoke).toHaveBeenCalledWith('create_branch', {
      path: '/repo',
      name: 'feat',
      commitId: null,
    });
  });

  it('createTag passes message when provided', async () => {
    await createTag('/repo', 'v1.0', 'abc123', 'Release 1.0');
    expect(invoke).toHaveBeenCalledWith('create_tag', {
      path: '/repo',
      name: 'v1.0',
      commitId: 'abc123',
      message: 'Release 1.0',
    });
  });

  it('createTag passes null when message omitted', async () => {
    await createTag('/repo', 'v1.0', 'abc123');
    expect(invoke).toHaveBeenCalledWith('create_tag', {
      path: '/repo',
      name: 'v1.0',
      commitId: 'abc123',
      message: null,
    });
  });

  it('readFileAtRef passes gitRef when provided', async () => {
    await readFileAtRef('/repo', 'src/main.ts', 'HEAD');
    expect(invoke).toHaveBeenCalledWith('read_file_at_ref', {
      path: '/repo',
      filePath: 'src/main.ts',
      gitRef: 'HEAD',
    });
  });

  it('readFileAtRef passes null when gitRef omitted', async () => {
    await readFileAtRef('/repo', 'src/main.ts');
    expect(invoke).toHaveBeenCalledWith('read_file_at_ref', {
      path: '/repo',
      filePath: 'src/main.ts',
      gitRef: null,
    });
  });

  it('setUpstream passes upstream when provided', async () => {
    await setUpstream('/repo', 'main', 'origin/main');
    expect(invoke).toHaveBeenCalledWith('set_upstream', {
      path: '/repo',
      branchName: 'main',
      upstream: 'origin/main',
    });
  });

  it('setUpstream passes null when upstream omitted', async () => {
    await setUpstream('/repo', 'main');
    expect(invoke).toHaveBeenCalledWith('set_upstream', {
      path: '/repo',
      branchName: 'main',
      upstream: null,
    });
  });

  it('stashSave passes includeUntracked when provided', async () => {
    await stashSave('/repo', 'WIP', true);
    expect(invoke).toHaveBeenCalledWith('stash_save', {
      path: '/repo',
      message: 'WIP',
      includeUntracked: true,
    });
  });

  it('stashSave passes null when includeUntracked omitted', async () => {
    await stashSave('/repo', 'WIP');
    expect(invoke).toHaveBeenCalledWith('stash_save', {
      path: '/repo',
      message: 'WIP',
      includeUntracked: null,
    });
  });
});

describe('optional Credentials commands (no ?? null)', () => {
  it('fetchRemote passes credentials when provided', async () => {
    const creds = { username: 'user', password: 'pass' };
    await fetchRemote('/repo', 'origin', creds);
    expect(invoke).toHaveBeenCalledWith('fetch_remote', {
      path: '/repo',
      remoteName: 'origin',
      credentials: creds,
    });
  });

  it('fetchRemote passes undefined when credentials omitted', async () => {
    await fetchRemote('/repo', 'origin');
    expect(invoke).toHaveBeenCalledWith('fetch_remote', {
      path: '/repo',
      remoteName: 'origin',
      credentials: undefined,
    });
  });
});

describe('many-param commands', () => {
  it('stageHunk passes all 8 parameters correctly', async () => {
    const lines = ['+added line', ' context', '-removed'];
    await stageHunk('/repo', 'file.ts', 10, 3, 10, 4, '@@ -10,3 +10,4 @@', lines);
    expect(invoke).toHaveBeenCalledWith('stage_hunk', {
      path: '/repo',
      filePath: 'file.ts',
      oldStart: 10,
      oldLines: 3,
      newStart: 10,
      newLines: 4,
      header: '@@ -10,3 +10,4 @@',
      lines,
    });
  });
});
