#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "error: $*" >&2
  exit 1
}

if [ "$#" -ne 2 ]; then
  fail "usage: scripts/release.sh <version> <r-version>"
fi

version="$1"
r_version="$2"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Validate the <r_version> exists in source/
[ -d "source/$r_version" ] || fail "source/$r_version does not exist"

# Assert <r_version> is the newest folder in source/ (numeric version sort)
newest="$(ls source | sort -V | tail -n1)"
[ "$r_version" = "$newest" ] || fail "$r_version is not the newest version in source/ ($newest)"

# Refuse to clobber an existing release
if gh release view "$version" >/dev/null 2>&1; then
  fail "release $version already exists"
fi

# Build the asset with the same code path as `just compress`
cargo xtask compress

# Create the release with the single asset, empty notes, marked latest
gh release create "$version" r-source.tar.zst --title "$version (R $r_version)" --notes "" --latest

# Clean up
rm -f r-source.tar.zst
