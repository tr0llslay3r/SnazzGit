export interface RepoInfo {
  path: string;
  name: string;
  current_branch: string | null;
  is_bare: boolean;
  branches: BranchInfo[];
  remotes: string[];
  tags: string[];
  stash_count: number;
}

export interface BranchInfo {
  name: string;
  is_head: boolean;
  is_remote: boolean;
  upstream: string | null;
  commit_id: string;
}

export interface CommitInfo {
  id: string;
  short_id: string;
  message: string;
  summary: string;
  author_name: string;
  author_email: string;
  author_time: number;
  committer_name: string;
  committer_time: number;
  parent_ids: string[];
  refs: RefInfo[];
}

export interface RefInfo {
  name: string;
  ref_type: 'LocalBranch' | 'RemoteBranch' | 'Tag' | 'Head';
}

export interface GraphRow {
  commit_id: string;
  column: number;
  edges: GraphEdge[];
  num_columns: number;
}

export interface GraphEdge {
  from_column: number;
  to_column: number;
  edge_type: 'Straight' | 'MergeLeft' | 'MergeRight' | 'ForkLeft' | 'ForkRight';
}

export interface FileStatus {
  path: string;
  status: 'New' | 'Modified' | 'Deleted' | 'Renamed' | 'Typechange' | 'Conflicted';
  old_path: string | null;
}

export interface WorkingTreeStatus {
  staged: FileStatus[];
  unstaged: FileStatus[];
  untracked: string[];
}

export interface DiffFile {
  path: string;
  hunks: DiffHunk[];
}

export interface DiffHunk {
  header: string;
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  line_type: 'Context' | 'Addition' | 'Deletion' | 'Header';
  content: string;
  old_lineno: number | null;
  new_lineno: number | null;
  spans: HighlightSpan[];
}

export interface HighlightSpan {
  start: number;
  end: number;
  style: string;
}

export interface StashEntry {
  index: number;
  message: string;
  commit_id: string;
}

export interface BlameInfo {
  lines: BlameLine[];
}

export interface BlameLine {
  line_number: number;
  commit_id: string;
  author: string;
  date: number;
  content: string;
}

export interface RecentRepo {
  path: string;
  name: string;
}

export interface Theme {
  name: string;
  colors: Record<string, string>;
}

export interface Toast {
  id: string;
  message: string;
  type: 'info' | 'success' | 'error' | 'warning';
  timeout?: number;
}
