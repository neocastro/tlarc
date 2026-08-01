//! The differential harness: drives Java TLC (the reference) and
//! `tlarc check --json` on the same spec + config, and emits a normalized
//! per-spec report.
//!
//! M0.6 scope: prove the plumbing — run both tools and report their
//! verdicts side by side. Semantic comparison of *checking* results lands
//! in M1, when tlarc can actually check. The report shape here is the
//! stable contract M1's parity checks extend.
//!
//! TLC quirks this module encodes (verified against the pinned
//! `tla2tools.jar` v1.7.4 / TLC 2.19 on 2026-08-01):
//!
//! - TLC must be given a unique `-metadir`: it derives a state directory
//!   from the current time and collides when two runs happen in the same
//!   second (`util.FileUtil.makeMetaDir` → `TLCRuntimeException`).
//! - The `SPECIFICATION` config directive fails on this build for plain
//!   action specs ("TLC cannot handle this conjunct of the spec"); corpus
//!   configs use the equivalent `INIT`/`NEXT` split form instead.
//! - TLC config files accept **no comments at all** — not even `%` — the
//!   parser fails with `ConfigFileException` on the first non-keyword line.
//!   Corpus `.cfg` files are pure directive lists.
//! - TLC's exit codes are raw JVM codes (e.g. 151 for "cannot handle this
//!   conjunct", 0 on success) — not a stable verdict contract. The output
//!   text is the source of truth for the verdict.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

/// How Java TLC itself judged the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TlcVerdict {
    /// "Model checking completed. No error has been found."
    NoError,
    /// At least one `Error:` line in TLC's output (semantic rejection,
    /// violated invariant, deadlock, …).
    Error,
    /// TLC ran but produced neither marker — treat as failure by callers.
    Unknown,
}

/// What Java TLC reported for one spec run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TlcOutcome {
    pub ran: bool,
    pub exit_code: i32,
    pub generated_states: Option<u64>,
    pub distinct_states: Option<u64>,
    pub verdict: TlcVerdict,
    /// The first `Error:` line from TLC, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// What `tlarc check --json` reported for one spec run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TlarcOutcome {
    pub ran: bool,
    pub exit_code: i32,
    /// The tlarc check verdict JSON (docs/cli.md), when the binary ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The normalized per-spec comparison report (the harness's unit of output).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpecReport {
    /// Spec base name, e.g. `"trivial"`.
    pub spec: String,
    pub tla: String,
    pub cfg: String,
    pub tlc: TlcOutcome,
    pub tlarc: TlarcOutcome,
    /// `true` iff both tools ran and accepted the spec: TLC with no error
    /// and exit 0, tlarc with an `"ok"` verdict.
    pub ok: bool,
}

/// A failure to invoke a tool — distinct from a tool rejecting a spec.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolError {
    pub message: String,
}

impl ToolError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parse a captured TLC output into an [`TlcOutcome`]. Pure and
/// unit-testable without a JVM.
pub fn parse_tlc_output(output: &str, exit_code: i32) -> TlcOutcome {
    let verdict = if output.contains("Model checking completed. No error has been found.") {
        TlcVerdict::NoError
    } else if output.contains("Error:") {
        TlcVerdict::Error
    } else {
        TlcVerdict::Unknown
    };

    let error = output
        .lines()
        .find(|line| line.contains("Error:"))
        .map(|line| line.trim().to_string());

    TlcOutcome {
        ran: true,
        exit_code,
        generated_states: number_before(output, " states generated,"),
        distinct_states: number_before(output, " distinct states found,"),
        verdict,
        error,
    }
}

/// The integer that immediately precedes `needle` in `haystack`, if any.
fn number_before(haystack: &str, needle: &str) -> Option<u64> {
    let idx = haystack.find(needle)?;
    let before = &haystack[..idx];
    before
        .rsplit(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|tok| tok.parse::<u64>().ok())
}

/// Run Java TLC on `spec` with `cfg` and capture its verdict.
///
/// Requires `java` on `PATH` and a `tla2tools.jar` at `jar`. A unique
/// temporary `-metadir` isolates each run (see module docs).
pub fn run_tlc(jar: &Path, spec: &Path, cfg: &Path) -> Result<TlcOutcome, ToolError> {
    let metadir = unique_metadir();
    let output = Command::new("java")
        .arg("-cp")
        .arg(jar)
        .arg("tlc2.TLC")
        .arg("-nowarning")
        .arg("-metadir")
        .arg(&metadir)
        .arg("-config")
        .arg(cfg)
        .arg(spec)
        .output()
        .map_err(|e| ToolError::new(format!("failed to spawn java (is it on PATH?): {e}")))?;
    let _ = std::fs::remove_dir_all(&metadir);

    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut outcome = parse_tlc_output(&text, output.status.code().unwrap_or(-1));
    if !output.status.success() {
        // Keep the parsed verdict, but record the non-zero exit.
        outcome.exit_code = output.status.code().unwrap_or(-1);
    }
    Ok(outcome)
}

/// Run `tlarc check --json` on `spec` with `cfg` and capture its verdict.
///
/// `tlarc_bin` is the path to the built `tlarc` binary. Even on a parse
/// error the CLI still emits the verdict JSON (docs/cli.md), so stdout is
/// parsed whenever it is valid JSON.
pub fn run_tlarc_check(
    tlarc_bin: &Path,
    spec: &Path,
    cfg: &Path,
) -> Result<TlarcOutcome, ToolError> {
    let output = Command::new(tlarc_bin)
        .arg("check")
        .arg("--json")
        .arg(spec)
        .arg(cfg)
        .output()
        .map_err(|e| {
            ToolError::new(format!(
                "failed to spawn tlarc binary '{}': {e}",
                tlarc_bin.display()
            ))
        })?;

    let text = String::from_utf8_lossy(&output.stdout);
    let verdict = serde_json::from_str::<serde_json::Value>(&text).ok();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let error = if output.status.success() {
        None
    } else {
        let from_stderr = stderr.trim();
        if !from_stderr.is_empty() {
            Some(from_stderr.to_string())
        } else {
            verdict
                .as_ref()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
        }
    };

    Ok(TlarcOutcome {
        ran: true,
        exit_code: output.status.code().unwrap_or(-1),
        verdict,
        error,
    })
}

/// Run both tools on one spec and produce the comparison report.
pub fn compare_spec(tlarc_bin: &Path, jar: &Path, tla: &Path, cfg: &Path) -> SpecReport {
    let tlc = run_tlc(jar, tla, cfg).unwrap_or(TlcOutcome {
        ran: false,
        exit_code: -1,
        generated_states: None,
        distinct_states: None,
        verdict: TlcVerdict::Unknown,
        error: None,
    });
    let tlarc = run_tlarc_check(tlarc_bin, tla, cfg).unwrap_or(TlarcOutcome {
        ran: false,
        exit_code: -1,
        verdict: None,
        error: None,
    });

    let tlc_ok = tlc.ran && tlc.exit_code == 0 && tlc.verdict == TlcVerdict::NoError;
    let tlarc_ok = tlarc.ran
        && tlarc.exit_code == 0
        && tlarc
            .verdict
            .as_ref()
            .and_then(|v| v.get("status").and_then(|s| s.as_str()))
            == Some("ok");

    SpecReport {
        spec: spec_base_name(tla),
        tla: tla.display().to_string(),
        cfg: cfg.display().to_string(),
        ok: tlc_ok && tlarc_ok,
        tlc,
        tlarc,
    }
}

/// The spec base name: `corpus/foo.tla` → `"foo"`.
fn spec_base_name(tla: &Path) -> String {
    tla.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| tla.display().to_string())
}

/// All `*.tla` files in `dir` that have a sibling `.cfg`, sorted by name
/// for deterministic output.
pub fn collect_specs(dir: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut specs = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return specs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("tla") {
            let cfg = path.with_extension("cfg");
            if cfg.exists() {
                specs.push((path, cfg));
            }
        }
    }
    specs.sort();
    specs
}

/// A fresh temporary directory for one TLC run's state files.
fn unique_metadir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("tlarc-tlc-{}-{nanos}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured from a real successful TLC run (corpus/trivial.tla).
    const SUCCESS_OUTPUT: &str = "\
TLC2 Version 2.19 of 08 August 2024 (rev: 5a47802)
Running breadth-first search Model-Checking with fp 93 and seed 2436805577566022128
Parsing file corpus/trivial.tla
Semantic processing of module trivial
Starting... (2026-08-01 15:16:44)
Model checking completed. No error has been found.
3 states generated, 2 distinct states found, 0 states left on queue.
The depth of the complete state graph search is 2.
Finished in 00s at (2026-08-01 15:16:44)";

    /// Captured from a real TLC run on a spec it cannot handle.
    const REJECT_OUTPUT: &str = "\
TLC2 Version 2.19 of 08 August 2024 (rev: 5a47802)
Parsing file /tmp/ok.tla
Semantic processing of module ok
Starting... (2026-08-01 15:17:54)
Error: TLC cannot handle this conjunct of the spec:
line 4, col 9 to line 4, col 14 of module ok
Finished in 00s at (2026-08-01 15:17:54)";

    #[test]
    fn parses_successful_run() {
        let o = parse_tlc_output(SUCCESS_OUTPUT, 0);
        assert!(o.ran);
        assert_eq!(o.exit_code, 0);
        assert_eq!(o.verdict, TlcVerdict::NoError);
        assert_eq!(o.generated_states, Some(3));
        assert_eq!(o.distinct_states, Some(2));
        assert_eq!(o.error, None);
    }

    #[test]
    fn parses_rejected_run() {
        let o = parse_tlc_output(REJECT_OUTPUT, 151);
        assert_eq!(o.verdict, TlcVerdict::Error);
        assert_eq!(o.generated_states, None);
        let err = o.error.expect("error line captured");
        assert!(err.contains("cannot handle this conjunct"), "{err}");
    }

    #[test]
    fn unknown_output_is_unknown() {
        let o = parse_tlc_output("jibberish\nno markers here", 3);
        assert_eq!(o.verdict, TlcVerdict::Unknown);
        assert_eq!(o.generated_states, None);
        assert_eq!(o.error, None);
    }

    #[test]
    fn state_counts_survive_other_numbers() {
        // Numbers appear in headers too; only the queue line counts.
        let o = parse_tlc_output(
            "with fp 93 and seed 2436805577566022128\n7 states generated, 5 distinct states found, 2 states left on queue.",
            0,
        );
        assert_eq!(o.generated_states, Some(7));
        assert_eq!(o.distinct_states, Some(5));
    }

    #[test]
    fn collects_specs_with_cfg_siblings() {
        let dir = std::env::temp_dir().join(format!("tlarc-corpus-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp corpus dir");
        std::fs::write(dir.join("a.tla"), "").expect("write");
        std::fs::write(dir.join("a.cfg"), "").expect("write");
        std::fs::write(dir.join("b.tla"), "").expect("write"); // no sibling cfg
        std::fs::write(dir.join("c.cfg"), "").expect("write"); // orphan cfg

        let specs = collect_specs(&dir);
        let names: Vec<String> = specs
            .iter()
            .map(|(tla, _)| tla.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.tla"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
