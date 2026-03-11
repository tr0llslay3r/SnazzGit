use std::collections::HashMap;

use super::types::{CommitInfo, EdgeType, GraphEdge, GraphRow};

pub fn compute_graph(commits: &[CommitInfo]) -> Vec<GraphRow> {
    let mut lanes: Vec<Option<String>> = Vec::new();
    let mut rows = Vec::new();
    let commit_index: HashMap<&str, usize> =
        commits.iter().enumerate().map(|(i, c)| (c.id.as_str(), i)).collect();

    for commit in commits {
        // Find which lane this commit occupies
        let column = lanes
            .iter()
            .position(|l| l.as_deref() == Some(&commit.id))
            .unwrap_or_else(|| {
                // New lane
                let pos = lanes.iter().position(|l| l.is_none()).unwrap_or(lanes.len());
                if pos == lanes.len() {
                    lanes.push(Some(commit.id.clone()));
                } else {
                    lanes[pos] = Some(commit.id.clone());
                }
                pos
            });

        let mut edges = Vec::new();

        // Snapshot lanes occupied before parent assignment (for continuation edges)
        let lanes_before: Vec<bool> = lanes.iter().map(|l| l.is_some()).collect();

        // Clear this lane
        lanes[column] = None;

        // Assign parents to lanes
        for (parent_idx, parent_id) in commit.parent_ids.iter().enumerate() {
            // Only assign lanes for parents that appear in our commit list
            if !commit_index.contains_key(parent_id.as_str()) {
                continue;
            }

            // Check if parent already has a lane
            let parent_lane = lanes
                .iter()
                .position(|l| l.as_deref() == Some(parent_id));

            let target_column = if let Some(lane) = parent_lane {
                lane
            } else if parent_idx == 0 {
                // First parent takes this commit's lane
                lanes[column] = Some(parent_id.clone());
                column
            } else {
                // Other parents get new lanes
                let pos = lanes.iter().position(|l| l.is_none()).unwrap_or(lanes.len());
                if pos == lanes.len() {
                    lanes.push(Some(parent_id.clone()));
                } else {
                    lanes[pos] = Some(parent_id.clone());
                }
                pos
            };

            let edge_type = if target_column == column {
                EdgeType::Straight
            } else if target_column < column {
                if parent_idx == 0 {
                    EdgeType::MergeLeft
                } else {
                    EdgeType::ForkLeft
                }
            } else if parent_idx == 0 {
                EdgeType::MergeRight
            } else {
                EdgeType::ForkRight
            };

            edges.push(GraphEdge {
                from_column: column,
                to_column: target_column,
                edge_type,
            });
        }

        // Add continuation edges for lanes that were already occupied before
        // this commit was processed (not lanes newly created by fork edges)
        for (i, lane) in lanes.iter().enumerate() {
            if lane.is_some() && i != column && i < lanes_before.len() && lanes_before[i] {
                edges.push(GraphEdge {
                    from_column: i,
                    to_column: i,
                    edge_type: EdgeType::Straight,
                });
            }
        }

        let num_columns = lanes.len();

        rows.push(GraphRow {
            commit_id: commit.id.clone(),
            column,
            edges,
            num_columns,
        });
    }

    // Trim trailing empty lanes from num_columns
    for row in &mut rows {
        while row.num_columns > 0 {
            let last = row.num_columns - 1;
            let used = row.edges.iter().any(|e| e.from_column == last || e.to_column == last)
                || row.column == last;
            if !used {
                row.num_columns -= 1;
            } else {
                break;
            }
        }
        row.num_columns = row.num_columns.max(row.column + 1);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(id: &str, parent_ids: Vec<&str>) -> CommitInfo {
        CommitInfo {
            id: id.to_string(),
            short_id: id[..id.len().min(8)].to_string(),
            message: String::new(),
            summary: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            author_time: 0,
            committer_name: String::new(),
            committer_time: 0,
            parent_ids: parent_ids.into_iter().map(|s| s.to_string()).collect(),
            refs: vec![],
        }
    }

    #[test]
    fn test_empty_input() {
        assert!(compute_graph(&[]).is_empty());
    }

    #[test]
    fn test_single_commit_no_parents() {
        let commits = vec![make_commit("aaaa", vec![])];
        let rows = compute_graph(&commits);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].column, 0);
        assert_eq!(rows[0].num_columns, 1);
        assert!(rows[0].edges.is_empty());
    }

    #[test]
    fn test_linear_chain() {
        let commits = vec![
            make_commit("child", vec!["parent"]),
            make_commit("parent", vec![]),
        ];
        let rows = compute_graph(&commits);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].column, 0);
        assert_eq!(rows[1].column, 0);
        // child has a single straight edge to parent
        assert_eq!(rows[0].edges.len(), 1);
        assert_eq!(rows[0].edges[0].from_column, 0);
        assert_eq!(rows[0].edges[0].to_column, 0);
        assert!(matches!(rows[0].edges[0].edge_type, EdgeType::Straight));
        // parent has no edges (not in the list as a commit's parent)
        assert!(rows[1].edges.is_empty());
    }

    #[test]
    fn test_linear_chain_three() {
        let commits = vec![
            make_commit("c", vec!["b"]),
            make_commit("b", vec!["a"]),
            make_commit("a", vec![]),
        ];
        let rows = compute_graph(&commits);
        assert_eq!(rows.len(), 3);
        for row in &rows {
            assert_eq!(row.column, 0);
            assert_eq!(row.num_columns, 1);
        }
    }

    #[test]
    fn test_merge_commit_two_parents() {
        let commits = vec![
            make_commit("merge", vec!["main_tip", "feature_tip"]),
            make_commit("main_tip", vec![]),
            make_commit("feature_tip", vec![]),
        ];
        let rows = compute_graph(&commits);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].column, 0);
        // merge row has edges for both parents
        assert_eq!(rows[0].edges.len(), 2);
        // first parent: straight (same lane)
        assert!(matches!(rows[0].edges[0].edge_type, EdgeType::Straight));
        // second parent: fork right (new lane)
        assert!(matches!(rows[0].edges[1].edge_type, EdgeType::ForkRight));
    }

    #[test]
    fn test_fork_two_branches_same_parent() {
        let commits = vec![
            make_commit("branch_a", vec!["common"]),
            make_commit("branch_b", vec!["common"]),
            make_commit("common", vec![]),
        ];
        let rows = compute_graph(&commits);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].column, 0);
        assert_eq!(rows[1].column, 1);
        assert_eq!(rows[2].column, 0);
    }

    #[test]
    fn test_commit_id_preserved() {
        let commits = vec![make_commit("myid123", vec![])];
        let rows = compute_graph(&commits);
        assert_eq!(rows[0].commit_id, "myid123");
    }
}
