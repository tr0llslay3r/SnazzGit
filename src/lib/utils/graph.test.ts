import { describe, it, expect } from 'vitest';
import {
  columnX,
  rowY,
  nodeRadius,
  edgePath,
  graphWidth,
  COLUMN_WIDTH,
  ROW_HEIGHT,
} from '$lib/utils/graph';
import type { GraphEdge } from '$lib/types';

describe('columnX', () => {
  it('returns COLUMN_WIDTH/2 for column 0', () => {
    expect(columnX(0)).toBe(COLUMN_WIDTH / 2);
  });

  it('returns correct x for column 1', () => {
    expect(columnX(1)).toBe(COLUMN_WIDTH + COLUMN_WIDTH / 2);
  });

  it('returns correct x for column 2', () => {
    expect(columnX(2)).toBe(2 * COLUMN_WIDTH + COLUMN_WIDTH / 2);
  });

  it('increases linearly with column index', () => {
    expect(columnX(3) - columnX(2)).toBe(COLUMN_WIDTH);
    expect(columnX(4) - columnX(3)).toBe(COLUMN_WIDTH);
  });
});

describe('rowY', () => {
  it('returns ROW_HEIGHT/2 for row 0', () => {
    expect(rowY(0)).toBe(ROW_HEIGHT / 2);
  });

  it('returns correct y for row 1', () => {
    expect(rowY(1)).toBe(ROW_HEIGHT + ROW_HEIGHT / 2);
  });

  it('increases linearly with row index', () => {
    expect(rowY(2) - rowY(1)).toBe(ROW_HEIGHT);
    expect(rowY(5) - rowY(4)).toBe(ROW_HEIGHT);
  });
});

describe('nodeRadius', () => {
  it('returns a positive number', () => {
    expect(nodeRadius()).toBeGreaterThan(0);
  });

  it('returns the same value every call', () => {
    expect(nodeRadius()).toBe(nodeRadius());
  });
});

describe('edgePath', () => {
  it('returns an SVG line for a straight (same-column) edge', () => {
    const edge: GraphEdge = { from_column: 0, to_column: 0, edge_type: 'Straight' };
    const path = edgePath(edge, 0, 1);
    expect(path).toMatch(/^M .+ L .+$/);
  });

  it('uses correct coordinates for straight edge', () => {
    const edge: GraphEdge = { from_column: 2, to_column: 2, edge_type: 'Straight' };
    const x = columnX(2);
    expect(edgePath(edge, 0, 1)).toBe(`M ${x} ${rowY(0)} L ${x} ${rowY(1)}`);
  });

  it('returns a bezier curve for edges between different columns', () => {
    const edge: GraphEdge = { from_column: 0, to_column: 1, edge_type: 'ForkRight' };
    const path = edgePath(edge, 0, 1);
    expect(path).toMatch(/^M .+ C .+$/);
  });

  it('bezier curve starts at from_column and ends at to_column', () => {
    const edge: GraphEdge = { from_column: 0, to_column: 2, edge_type: 'MergeRight' };
    const path = edgePath(edge, 0, 1);
    const x1 = columnX(0);
    const x2 = columnX(2);
    const y1 = rowY(0);
    const y2 = rowY(1);
    expect(path).toContain(`M ${x1} ${y1}`);
    expect(path).toContain(`${x2} ${y2}`);
  });
});

describe('graphWidth', () => {
  it('returns COLUMN_WIDTH for 0 columns', () => {
    expect(graphWidth(0)).toBe(COLUMN_WIDTH);
  });

  it('returns 2*COLUMN_WIDTH for 1 column', () => {
    expect(graphWidth(1)).toBe(2 * COLUMN_WIDTH);
  });

  it('returns correct width for 3 columns', () => {
    expect(graphWidth(3)).toBe(4 * COLUMN_WIDTH);
  });

  it('scales linearly', () => {
    expect(graphWidth(4) - graphWidth(3)).toBe(COLUMN_WIDTH);
  });
});
