#!/bin/sh
# Fails unless the release tag (minus its leading "v") equals the Cargo.toml
# package version, so `tickets --version` always matches the release name.
# Usage: check-release-tag.sh <tag> [path/to/Cargo.toml]
set -eu

tag=${1:?usage: check-release-tag.sh <tag> [Cargo.toml]}
manifest=${2:-Cargo.toml}

# The first top-level `version = "..."` line is the [package] version.
version=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' "$manifest" | head -n 1)
if [ -z "$version" ]; then
  echo "error: no package version found in $manifest" >&2
  exit 1
fi

if [ "${tag#v}" != "$version" ]; then
  echo "error: tag $tag does not match the Cargo.toml version $version" >&2
  exit 1
fi
echo "ok: tag $tag matches Cargo.toml version $version"
