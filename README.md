# Oak's R sources

Stripped down base R sources intended for usage inside of the oak engine inside of [ark](https://github.com/posit-dev/ark).

# Developers

To add a new R version, say 4.7.0:

```
# Downloads R 4.7.0's stripped down base package sources
just download 4.7.0

# Commit and push!

# Creates a GitHub Release with version 4.7.0.
# Uploads a compressed `r-source.tar.zst` as a release Asset.
just release 4.7.0
```

Update the Ark side to recognize 4.7.0 as the latest R version that we have sources for.

Ark will then download this Asset via this URL: <https://github.com/posit-dev/oak-r-sources/releases/download/4.7.0/r-source.tar.zst>.

The compressed `r-source.tar.zst` contains the sources for all R versions from 4.2.0 up to the current R version. See `xtask/src/compress.rs` for how we managed to shrink this into a ~2 MB blob.