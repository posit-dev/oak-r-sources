//! Compress the vendored base R sources into a solid `r-source.tar.zst`
//!
//! To maximally reduce duplication across R versions, we leverage zstd's compression
//! window feature. The compression algorithm "remembers" a window of previous data to
//! find and compress repeating patterns. Since there are very few changes between R
//! versions in the R sources, we can utilize this by placing `4.4.0/utils/R/help.R`
//! directly adjacent to `4.5.0/utils/R/help.R` in the tar file. This allows us to pack
//! the R sources for all supported patch releases of R into a single small `.tar.zst`.
//!
//! We've pinned down some zstd options:
//!
//! - Compression level 19. Chosen because it is the maximum "normal" amount of
//!   compression. Compression time is paid once on our end, and decompression is fast for
//!   users.
//!
//! - Window log 23 (2^23 = 8 MB of lookback memory). This ends up being the default
//!   with compression level 19 and a large file, but we chose it on purpose. Some local
//!   testing showed that we don't need `--long` (implying window log 27, 128 MB) to
//!   maximize compression gains here. Going from 23 to 27 only compressed a further ~4%,
//!   which is great! It means that we can limit the memory required to decompress on the
//!   user's machine to 8 MB, while still getting maximal compression.
//!
//! The `.tar.zst` is also byte-reproducible across calls with the same `source/`. We
//! accomplish this by normalizing all tar header data, (including mtime, uid, gid, and
//! file permissions) and having a fixed zstd compression level and window log.

use std::path::Path;

use crate::parse_version;
use crate::repo_root;
use crate::source_dir;

/// zstd compression level
const ZSTD_LEVEL: i32 = 19;

/// zstd window log (2^23 = 8 MB of memory required to decompress)
const ZSTD_WINDOW_LOG: u32 = 23;

/// One file kept from `source/`, ready to place in the solid archive
struct Entry {
    /// Version this file came from, as the original `x.y.z` string
    version: String,
    /// Version this file came from, parsed into (major, minor, patch) for ordering
    version_number: (u32, u32, u32),
    /// Path without the leading `{version}/`, e.g. `utils/R/help.R`
    path: String,
    /// File contents
    data: Vec<u8>,
}

/// Compress everything in `source/` into `r-source.tar.zst` at the repo root
pub fn run() -> anyhow::Result<()> {
    let mut entries = collect_entries(&source_dir())?;

    // Sort all entries by `(path, version_number)`, for example, `("utils/R/help.R",
    // "4.5.0")`. This places all versions of a given file adjacent to each other in the
    // tar file, allowing zstd's compression window feature to dedup identical copies
    // across versions.
    entries.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.version_number.cmp(&right.version_number))
    });

    let output = repo_root().join("r-source.tar.zst");
    write_archive(&entries, &output)?;

    eprintln!(
        "Wrote {count} files to {output}",
        count = entries.len(),
        output = output.display()
    );

    Ok(())
}

/// Collect every file under `source/{version}/` as an [`Entry`]
fn collect_entries(source: &Path) -> anyhow::Result<Vec<Entry>> {
    let mut entries = Vec::new();

    for version_dir in std::fs::read_dir(source)? {
        let version_dir = version_dir?;

        if !version_dir.file_type()?.is_dir() {
            continue;
        }

        let version = version_dir
            .file_name()
            .into_string()
            .map_err(|name| anyhow::anyhow!("Non-UTF-8 version directory: {name:?}"))?;

        let version_number = parse_version(&version)
            .ok_or_else(|| anyhow::anyhow!("Invalid R version: {version}"))?;

        let root = version_dir.path();

        for entry in walkdir::WalkDir::new(&root) {
            let entry = entry?;

            if !entry.file_type().is_file() {
                continue;
            }

            let relative = entry.path().strip_prefix(&root)?;
            let path = normalize_path(relative).ok_or_else(|| {
                anyhow::anyhow!("Non-UTF-8 path: {relative}", relative = relative.display())
            })?;
            let data = std::fs::read(entry.path())?;

            entries.push(Entry {
                version: version.clone(),
                version_number,
                path,
                data,
            });
        }
    }

    Ok(entries)
}

/// Join a relative path's components with `/`, returning `None` on non-UTF-8 input
///
/// For cross OS consistency, and because the tar crate prefers unix separators
fn normalize_path(path: &Path) -> Option<String> {
    let mut out = String::new();

    for component in path.components() {
        let component = component.as_os_str().to_str()?;
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(component);
    }

    Some(out)
}

/// Write the sorted entries to a solid `tar.zst` archive reproducibly
fn write_archive(entries: &[Entry], output: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::create(output)?;

    let mut encoder = zstd::stream::write::Encoder::new(file, ZSTD_LEVEL)?;
    encoder.set_parameter(zstd::zstd_safe::CParameter::WindowLog(ZSTD_WINDOW_LOG))?;

    let mut builder = tar::Builder::new(encoder);

    for entry in entries {
        let path = format!(
            "{version}/{path}",
            version = entry.version,
            path = entry.path
        );

        // Start with a blank GNU header (the default for the tar crate)
        let mut header = tar::Header::new_gnu();

        // Write reproducible header information
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(entry.data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);

        // Finalize checksum
        header.set_cksum();

        // Write it!
        builder.append_data(&mut header, path, entry.data.as_slice())?;
    }

    let encoder = builder.into_inner()?;
    encoder.finish()?;

    Ok(())
}
