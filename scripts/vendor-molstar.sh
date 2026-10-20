#!/usr/bin/env bash
# Build Mol* v6 from the GitHub head and vendor the packages Fennec uses.
#
# Mol* v6 (6.0.0-dev) is a pnpm workspace that is not published to npm yet, and npm cannot
# install a package from a subdirectory of a git repository. So we build the workspace,
# pack the public packages, and install the tarballs from vendor/molstar/ (see package.json).
#
# Usage: scripts/vendor-molstar.sh [commit]   (default: the pinned commit below)
# Requires: node >= 22.13, pnpm (the version in Mol*'s packageManager field), git.
set -euo pipefail

MOLSTAR_COMMIT="${1:-29cd9cd379e606c9516601d3ad21639be6b53b36}"
PACKAGES=(core io model graphics query-language plugin)

root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/vendor/molstar"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

git -C "$work" init -q molstar
git -C "$work/molstar" fetch -q --depth 1 https://github.com/molstar/molstar.git "$MOLSTAR_COMMIT"
git -C "$work/molstar" checkout -q FETCH_HEAD

(
  cd "$work/molstar"
  pnpm install --frozen-lockfile
  pnpm build:lib
  pnpm pack:workspace
)

version="$(node -p "require('$work/molstar/version.json').version")"
rm -f "$dest"/*.tgz
mkdir -p "$dest"
for p in "${PACKAGES[@]}"; do
  cp "$work/molstar/build/packages/molstar-$p-$version.tgz" "$dest/"
done
echo "$MOLSTAR_COMMIT" > "$dest/COMMIT"
echo "Vendored Mol* $version @ $MOLSTAR_COMMIT into $dest"
echo "If the version changed, update the file: specs in package.json and run npm install."
