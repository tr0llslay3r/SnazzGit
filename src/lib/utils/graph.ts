import type { GraphEdge } from '$lib/types';

const COLUMN_WIDTH = 16;
const ROW_HEIGHT = 28;
const NODE_RADIUS = 4;

export function columnX(column: number): number {
  return column * COLUMN_WIDTH + COLUMN_WIDTH / 2;
}

export function rowY(rowIndex: number): number {
  return rowIndex * ROW_HEIGHT + ROW_HEIGHT / 2;
}

export function nodeRadius(): number {
  return NODE_RADIUS;
}

export function edgePath(
  edge: GraphEdge,
  fromRowIndex: number,
  toRowIndex: number,
): string {
  const x1 = columnX(edge.from_column);
  const y1 = rowY(fromRowIndex);
  const x2 = columnX(edge.to_column);
  const y2 = rowY(toRowIndex);

  if (x1 === x2) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  // Bezier curve for merges/forks
  const midY = (y1 + y2) / 2;
  return `M ${x1} ${y1} C ${x1} ${midY}, ${x2} ${midY}, ${x2} ${y2}`;
}

export function graphWidth(numColumns: number): number {
  return numColumns * COLUMN_WIDTH + COLUMN_WIDTH;
}

export { COLUMN_WIDTH, ROW_HEIGHT };
