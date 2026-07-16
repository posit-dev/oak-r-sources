use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use flate2::read::GzDecoder;

use crate::parse_version;
use crate::source_dir;

/// Download the `src/library/*/R/` subtree of R `version` into `source/{version}/`
pub fn run(version: &str) -> anyhow::Result<()> {
    parse_version(version).ok_or_else(|| anyhow::anyhow!("Invalid R version: {version}"))?;

    eprintln!("Downloading R {version}");
    let tarball = download(version)?;

    let destination = source_dir().join(version);
    if destination.exists() {
        std::fs::remove_dir_all(&destination)?;
    }

    let count = extract_r_sources(&tarball, &destination)?;

    eprintln!(
        "Wrote {count} files to {destination}",
        destination = destination.display()
    );

    Ok(())
}

/// Download the R source tarball for `version`, returning its gzipped bytes
fn download(version: &str) -> anyhow::Result<Vec<u8>> {
    /// CRAN mirrors to try, in order
    const MIRRORS: &[&str] = &["https://cran.rstudio.com", "https://cran.r-project.org"];

    const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    const GLOBAL_TIMEOUT: Duration = Duration::from_secs(300);

    let major = version
        .split('.')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid R version: {version}"))?;

    let suffix = format!("src/base/R-{major}/R-{version}.tar.gz");

    let mut last_error = None;

    for mirror in MIRRORS {
        let url = format!("{mirror}/{suffix}");

        let request = ureq::get(&url)
            .config()
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(GLOBAL_TIMEOUT))
            .build();

        match request.call() {
            Ok(response) => {
                let mut bytes = Vec::new();
                response.into_body().into_reader().read_to_end(&mut bytes)?;
                return Ok(bytes);
            }
            Err(err) => {
                last_error = Some(err);
                continue;
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to download R {version}: {err:?}",
        err = last_error.expect("`MIRRORS` is non-empty")
    ))
}

/// Extract the `src/library/*/R/` subtree from an R source tarball into `destination`
fn extract_r_sources(tarball: &[u8], destination: &Path) -> anyhow::Result<usize> {
    let gz = GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(gz);

    // Parent directories we've already created
    let mut created: HashSet<PathBuf> = HashSet::new();
    let mut count = 0;

    for entry in archive.entries()? {
        let mut entry = entry?;

        if !entry.header().entry_type().is_file() {
            continue;
        }

        let path = entry.path()?.into_owned();
        let Some(path) = detect_package_r_file(&path) else {
            continue;
        };

        let target = destination.join(path);

        // We must create parent directories before unpacking into them. We remember ones
        // we've already created to avoid thousands of redundant `create_dir_all()` calls.
        if let Some(parent) = target.parent() {
            if !created.contains(parent) {
                std::fs::create_dir_all(parent)?;
                created.insert(parent.to_path_buf());
            }
        }

        entry.unpack(&target)?;
        count += 1;
    }

    Ok(count)
}

/// Detect files that live under `R-{version}/src/library/{package}/R/{rest}` and return
/// their destination path of `{package}/R/{rest}`
///
/// For some base packages, like parallel, `{rest}` can be another subfolder, like
/// `parallel/R/unix/{file}.R`.
///
/// Anything that doesn't live under that file path returns `None`
fn detect_package_r_file(path: &Path) -> Option<String> {
    let components: Vec<&str> = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;

    // R-{version} / src / library / {package} / R / <rest...>
    let [_r_version, "src", "library", package, "R", rest @ ..] = components.as_slice() else {
        return None;
    };

    // We need a file!
    if rest.is_empty() {
        return None;
    }

    let mut destination = format!("{package}/R");

    for part in rest {
        destination.push('/');
        destination.push_str(part);
    }

    Some(destination)
}
