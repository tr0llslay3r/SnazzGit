use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub name: String,
    pub current_branch: Option<String>,
    pub is_bare: bool,
    pub branches: Vec<BranchInfo>,
    pub remotes: Vec<String>,
    pub tags: Vec<String>,
    pub stash_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub commit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub committer_name: String,
    pub committer_time: i64,
    pub parent_ids: Vec<String>,
    pub refs: Vec<RefInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefInfo {
    pub name: String,
    pub ref_type: RefType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefType {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRow {
    pub commit_id: String,
    pub column: usize,
    pub edges: Vec<GraphEdge>,
    pub num_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_column: usize,
    pub to_column: usize,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Straight,
    MergeLeft,
    MergeRight,
    ForkLeft,
    ForkRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub old_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatusType {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingTreeStatus {
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub spans: Vec<HighlightSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub commit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameInfo {
    pub lines: Vec<BlameLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameLine {
    pub line_number: u32,
    pub commit_id: String,
    pub author: String,
    pub date: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictHunkKind {
    Context,
    Ours,
    Theirs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictHunk {
    pub kind: ConflictHunkKind,
    pub lines: Vec<String>,
    pub start_line: u32,
    pub conflict_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFile {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub our_label: String,
    pub their_label: String,
}
