//! M0.6 end-to-end tests: run the real `tlarc-test-harness` binary against
//! Java TLC and `tlarc` on the corpus.
//!
//! Like `tests/m05_e2e.rs`, these skip when the bridge toolchain is
//! unavailable locally; CI provisions it (see .github/workflows/ci.yml) so
//! they run for real there.

use std::path::PathBuf;
use std::process::Command;

fn harness() -> &'static str {
    env!("CARGO_BIN_EXE_tlarc-test-harness")
}

fn tlarc_bin() -> &'static str {
    env!("CARGO_BIN_EXE_tlarc")
}

/// The jar must be present and `java` on PATH; otherwise the e2e can't run.
fn toolchain_available() -> Result<(), String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !manifest.join("bridge/lib/tla2tools.jar").exists() {
        return Err("tla2tools.jar missing — run scripts/fetch-tla2tools.sh".into());
    }
    let java = Command::new("java").arg("-version").output();
    if java.map(|o| !o.status.success()).unwrap_or(true) {
        return Err("java not on PATH".into());
    }
    Ok(())
}

fn corpus(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join(path)
}

fn run_harness(corpus_dir: &PathBuf) -> std::process::Output {
    Command::new(harness())
        .arg("--corpus")
        .arg(corpus_dir)
        .arg("--tlarc-bin")
        .arg(tlarc_bin())
        .output()
        .expect("harness must spawn")
}

#[test]
fn e2e_harness_green_on_corpus() {
    if let Err(reason) = toolchain_available() {
        eprintln!("skipping e2e (toolchain unavailable): {reason}");
        return;
    }

    let out = run_harness(&corpus("."));
    assert!(
        out.status.success(),
        "harness must exit 0 on the corpus, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("harness stdout is the JSON report");
    assert_eq!(report["tool"], "tlarc-test-harness");
    assert_eq!(report["passed"], 1);
    assert_eq!(report["failed"], 0);

    let spec = &report["specs"][0];
    assert_eq!(spec["spec"], "trivial");
    assert_eq!(spec["ok"], true);
    assert_eq!(spec["tlc"]["ran"], true);
    assert_eq!(spec["tlc"]["verdict"], "no_error");
    assert_eq!(spec["tlc"]["exit_code"], 0);
    assert_eq!(spec["tlc"]["distinct_states"], 2);
    assert_eq!(spec["tlarc"]["ran"], true);
    assert_eq!(spec["tlarc"]["exit_code"], 0);
    assert_eq!(spec["tlarc"]["verdict"]["status"], "ok");
}

#[test]
fn e2e_harness_exits_nonzero_on_tlc_rejected_spec() {
    if let Err(reason) = toolchain_available() {
        eprintln!("skipping e2e (toolchain unavailable): {reason}");
        return;
    }

    // A spec TLC rejects: an invariant that does not hold. (Verified
    // 2026-08-01: with the INIT/NEXT split config, TLC 2.19 model-checks
    // `x' = IF ...` fine — the SPECIFICATION directive was the blocker.)
    // The harness must report the rejection and exit non-zero.
    let dir = std::env::temp_dir().join(format!("tlarc-harness-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp corpus dir");
    std::fs::write(
        dir.join("bad.tla"),
        "---- MODULE bad ----\nVARIABLE x\nInit == x = 0\nNext == x' = IF x = 0 THEN 1 ELSE 0\nInv == x = 5\n====\n",
    )
    .expect("write bad spec");
    std::fs::write(dir.join("bad.cfg"), "INIT Init\nNEXT Next\nINVARIANT Inv\n")
        .expect("write bad cfg");

    let out = run_harness(&dir);
    assert!(
        !out.status.success(),
        "harness must exit non-zero when TLC rejects a corpus spec"
    );

    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("harness stdout is the JSON report");
    assert_eq!(report["failed"], 1);
    assert_eq!(report["specs"][0]["ok"], false);
    assert_eq!(report["specs"][0]["tlc"]["verdict"], "error");
    assert!(
        report["specs"][0]["tlc"]["error"]
            .as_str()
            .unwrap_or("")
            .contains("violated"),
        "TLC error line must be captured in the report"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn e2e_harness_fails_fast_without_jar() {
    // No toolchain needed: a missing jar is a harness misconfiguration.
    let dir = std::env::temp_dir().join(format!("tlarc-harness-nojar-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp corpus dir");
    std::fs::write(dir.join("s.tla"), "---- MODULE s ----\n====\n").expect("write spec");
    std::fs::write(dir.join("s.cfg"), "INIT Init\n").expect("write cfg");

    let out = Command::new(harness())
        .arg("--corpus")
        .arg(&dir)
        .arg("--tlarc-bin")
        .arg(tlarc_bin())
        .arg("--jar")
        .arg("/nonexistent/tla2tools.jar")
        .output()
        .expect("harness must spawn");

    assert!(!out.status.success(), "missing jar must fail fast");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("tla2tools.jar not found"),
        "stderr: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
