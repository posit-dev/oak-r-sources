# Download the base R sources for <version> from CRAN into `source/`
download version:
  cargo xtask download {{version}}

# Compress `source/` into a reproducible `r-source.tar.zst` at the repo root
compress:
  cargo xtask compress

# Cut a GitHub Release for <version> with `r-source.tar.zst` as its asset
release version:
  scripts/release.sh {{version}}
