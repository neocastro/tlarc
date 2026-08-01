//! `tlarc-test-harness` — the M0.6 differential harness (PLAN.md).
//!
//! Drives Java TLC (the reference) and `tlarc check --json` over every
//! spec in a corpus directory and emits one normalized JSON report.
//! Exits non-zero if any corpus spec is rejected by either tool, or if a
//! tool cannot be run at all.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use serde::Serialize;
use tlarc::diff::{collect_specs, compare_spec, SpecReport};

/// The full harness report: one entry per spec plus a summary.
#[derive(Debug, Serialize)]
struct HarnessReport {
    tool: &'static str,
    version: &'static str,
    corpus: String,
    specs: Vec<SpecReport>,
    passed: usize,
    failed: usize,
}

#[derive(Parser, Debug)]
#[command(name = "tlarc-test-harness", version, about)]
struct Args {
    /// Corpus directory containing `*.tla` specs with sibling `.cfg` files.
    #[arg(long, default_value = "corpus")]
    corpus: PathBuf,

    /// Path to the built `tlarc` binary. Defaults to `TLARC_BIN`, then a
    /// sibling `tlarc` next to this binary, then `tlarc` on PATH.
    #[arg(long)]
    tlarc_bin: Option<PathBuf>,

    /// Path to `tla2tools.jar`. Defaults to `TLARC_TLC_JAR`, then
    /// `$CARGO_MANIFEST_DIR/bridge/lib/tla2tools.jar`.
    #[arg(long)]
    jar: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let jar = resolve_jar(args.jar.as_deref());
    let tlarc_bin = resolve_tlarc_bin(args.tlarc_bin.as_deref());

    // A missing tool is a harness misconfiguration, not a spec verdict:
    // fail fast with actionable guidance instead of a per-spec report.
    if !jar.exists() {
        eprintln!(
            "tlarc-test-harness: tla2tools.jar not found at '{}' — \
             run scripts/fetch-tla2tools.sh first, or pass --jar",
            jar.display()
        );
        return ExitCode::FAILURE;
    }
    if !tlarc_bin.exists() {
        eprintln!(
            "tlarc-test-harness: tlarc binary not found at '{}' — \
             pass --tlarc-bin, or set TLARC_BIN",
            tlarc_bin.display()
        );
        return ExitCode::FAILURE;
    }

    let specs = collect_specs(&args.corpus);
    if specs.is_empty() {
        eprintln!(
            "tlarc-test-harness: no *.tla specs with sibling .cfg found in '{}'",
            args.corpus.display()
        );
        return ExitCode::FAILURE;
    }

    let reports: Vec<SpecReport> = specs
        .iter()
        .map(|(tla, cfg)| compare_spec(&tlarc_bin, &jar, tla, cfg))
        .collect();

    let passed = reports.iter().filter(|r| r.ok).count();
    let failed = reports.len() - passed;

    let report = HarnessReport {
        tool: "tlarc-test-harness",
        version: tlarc::VERSION,
        corpus: args.corpus.display().to_string(),
        specs: reports,
        passed,
        failed,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("report is serializable")
    );

    if failed > 0 {
        eprintln!(
            "tlarc-test-harness: {failed} of {} specs failed",
            passed + failed
        );
        ExitCode::FAILURE
    } else {
        eprintln!("tlarc-test-harness: {passed} specs passed");
        ExitCode::SUCCESS
    }
}

/// The jar path: explicit arg, then `TLARC_TLC_JAR`, then the repo default.
fn resolve_jar(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    if let Ok(path) = std::env::var("TLARC_TLC_JAR") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(tlarc::TLA2TOOLS_JAR_RELPATH)
}

/// The tlarc binary path: explicit arg, then `TLARC_BIN`, then a sibling
/// `tlarc` next to this harness, then `tlarc` on PATH.
fn resolve_tlarc_bin(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    if let Ok(path) = std::env::var("TLARC_BIN") {
        return PathBuf::from(path);
    }
    if let Ok(exe) = std::env::current_exe() {
        let sibling = exe.with_file_name("tlarc");
        if sibling.exists() {
            return sibling;
        }
    }
    PathBuf::from("tlarc")
}
