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
            } else {
                if parent_idx == 0 {
                    EdgeType::MergeRight
                } else {
                    EdgeType::ForkRight
                }
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
