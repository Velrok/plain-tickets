#!/usr/bin/env bash
# Bump Cargo.toml version, commit, tag, and push to trigger release.yml.
# Usage: scripts/release.sh <version>   e.g. scripts/release.sh 0.2.0
set -euo pipefail

version="${1:-}"
if [[ -z "$version" ]]; then
  echo "Usage: $0 <version>   e.g. $0 0.2.0" >&2
  exit 1
fi
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be X.Y.Z (got: $version)" >&2
  exit 1
fi

tag="v$version"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Error: working tree is not clean" >&2
  exit 1
fi

branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$branch" != "main" ]]; then
  echo "Error: must be on main (currently on $branch)" >&2
  exit 1
fi

git fetch origin main
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
  echo "Error: local main is not up to date with origin/main" >&2
  exit 1
fi

if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  echo "Error: tag $tag already exists" >&2
  exit 1
fi

echo "==> Running fmt, clippy, test"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

echo "==> Bumping version to $version"
sed -i.bak -E "0,/^version = \".*\"/s//version = \"$version\"/" Cargo.toml
rm -f Cargo.toml.bak
cargo build >/dev/null # refresh Cargo.lock

git add Cargo.toml Cargo.lock
if git diff --cached --quiet; then
  echo "==> Cargo.toml already at $version, nothing to commit"
else
  git commit -m "chore: release $tag"
fi
git tag -a "$tag" -m "$tag"

read -r -p "Push commit and tag $tag to origin? [y/N] " confirm
if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
  echo "Not pushing. Undo with: git tag -d $tag && git reset --hard HEAD~1"
  exit 0
fi

git push origin main
git push origin "$tag"

echo "==> Pushed $tag — release.yml will build and publish the release"
