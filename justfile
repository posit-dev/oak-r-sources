# Download the base R sources for <version> from CRAN into `source/`
download version:
  cargo xtask download {{version}}

# Compress `source/` into a reproducible `r-source.tar.zst` at the repo root
compress:
  cargo xtask compress

# Cut a GitHub Release for <version> with `r-source.tar.zst` as its asset.
# <version> should be v1, v2, etc.
# Validates that <r-version> is the most recent folder in `source/` before releasing.
release version r-version:
  scripts/release.sh {{version}} {{r-version}}
