# Oak's R sources

Stripped down base R sources intended for usage inside of the oak engine inside of [ark](https://github.com/posit-dev/ark).

# Developers

To add a new R version, say 4.7.0:

```
# Downloads R 4.7.0's stripped down base package sources
just download 4.7.0

# Commit and push!

# Creates a GitHub Release with version `v2` for R 4.7.0 and earlier.
# Uploads a compressed `r-source.tar.zst` as a release Asset.
just release v2 4.7.0
```

Bump the GitHub Release version by 1 between releases, i.e. `v2` to `v3`. You'll need to check and see what the most recent release version is and do 1 beyond that. Keeping the release version separate from the R version lets us fix any mistakes or make changes to releases separate from new R releases.

On the Ark side, update `OAK_R_SOURCES_ASSET_VERSION` and `OAK_R_SOURCES_LATEST_R_VERSION` to reflect the latest release.

Ark will then download this Asset via this URL: <https://github.com/posit-dev/oak-r-sources/releases/download/v2/r-source.tar.zst>.

The compressed `r-source.tar.zst` contains the sources for all R versions from 4.2.0 up to the current R version. See `xtask/src/compress.rs` for how we managed to shrink this into a ~2 MB blob.