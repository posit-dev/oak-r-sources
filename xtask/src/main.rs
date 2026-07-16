mod compress;
mod download;

use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Download the base R sources for a version from CRAN into `source/`
    Download { version: String },

    /// Compress `source/` into a reproducible `r-source.tar.zst`
    Compress,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Download { version } => download::run(&version),
        Command::Compress => compress::run(),
    }
}

/// Repository root, one level up from this `xtask` crate
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("`xtask` always has a parent")
        .to_path_buf()
}

/// The `source/` directory that holds the vendored R sources
fn source_dir() -> PathBuf {
    repo_root().join("source")
}

/// Parse `x.y.z` into a comparable tuple
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.split('.');

    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;

    if parts.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}
