// Word-level diff highlighting for inline changes within modified lines

export interface WordDiffSegment {
  text: string;
  changed: boolean;
}

// Simple word-level diff using longest common subsequence on words
export function computeWordDiff(
  oldLine: string,
  newLine: string
): { oldSegments: WordDiffSegment[]; newSegments: WordDiffSegment[] } {
  const oldWords = tokenize(oldLine);
  const newWords = tokenize(newLine);

  const lcs = longestCommonSubsequence(oldWords, newWords);
  const oldSegments = buildSegments(oldWords, lcs, 'old');
  const newSegments = buildSegments(newWords, lcs, 'new');

  return { oldSegments, newSegments };
}

function tokenize(line: string): string[] {
  // Split into words and whitespace, preserving everything
  return line.match(/\S+|\s+/g) ?? [];
}

function longestCommonSubsequence(a: string[], b: string[]): Set<string> {
  const m = a.length;
  const n = b.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (a[i - 1] === b[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to find which indices are in LCS
  const lcsIndicesA = new Set<number>();
  const lcsIndicesB = new Set<number>();
  let i = m,
    j = n;
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) {
      lcsIndicesA.add(i - 1);
      lcsIndicesB.add(j - 1);
      i--;
      j--;
    } else if (dp[i - 1][j] > dp[i][j - 1]) {
      i--;
    } else {
      j--;
    }
  }

  return new Set([...lcsIndicesA].map((idx) => `a:${idx}`).concat([...lcsIndicesB].map((idx) => `b:${idx}`)));
}

function buildSegments(
  words: string[],
  lcs: Set<string>,
  side: 'old' | 'new'
): WordDiffSegment[] {
  const prefix = side === 'old' ? 'a' : 'b';
  const segments: WordDiffSegment[] = [];
  let currentText = '';
  let currentChanged: boolean | null = null;

  for (let i = 0; i < words.length; i++) {
    const inLcs = lcs.has(`${prefix}:${i}`);
    const changed = !inLcs;

    if (currentChanged !== null && changed !== currentChanged) {
      segments.push({ text: currentText, changed: currentChanged });
      currentText = '';
    }
    currentText += words[i];
    currentChanged = changed;
  }

  if (currentText) {
    segments.push({ text: currentText, changed: currentChanged ?? false });
  }

  return segments;
}
