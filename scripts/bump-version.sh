#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: ./scripts/bump-version.sh <version>"
  echo "Example: ./scripts/bump-version.sh 0.2.0"
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Cargo.toml — update the first version = line (in [package])
sed -i "0,/^version = /s/^version = .*/version = \"$VERSION\"/" "$REPO_ROOT/src-tauri/Cargo.toml"

# package.json
cd "$REPO_ROOT" && npm version "$VERSION" --no-git-tag-version --allow-same-version

# tauri.conf.json
jq --arg v "$VERSION" '.version = $v' "$REPO_ROOT/src-tauri/tauri.conf.json" > "$REPO_ROOT/src-tauri/tauri.conf.json.tmp" \
  && mv "$REPO_ROOT/src-tauri/tauri.conf.json.tmp" "$REPO_ROOT/src-tauri/tauri.conf.json"

echo "Version bumped to $VERSION in:"
echo "  - src-tauri/Cargo.toml"
echo "  - package.json"
echo "  - src-tauri/tauri.conf.json"
echo ""
echo "Next steps:"
echo "  git add -A && git commit -m 'chore: bump version to $VERSION'"
echo "  git tag v$VERSION"
echo "  git push origin main --tags"
