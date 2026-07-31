//! M0.5 end-to-end tests: run the real `sany-json` bridge via the tlarc
//! library and exercise both the success and the error path.
//!
//! These tests require the bridge toolchain (a JVM, `tla2tools.jar`, and the
//! compiled bridge classes). When any prerequisite is missing they skip with
//! a message — CI provisions them (see .github/workflows/ci.yml) so they run
//! for real there.

use std::path::PathBuf;
use std::process::Command;

/// The bridge runner must exist; java must be on PATH; the jar and compiled
/// classes must be present. Returns Ok(()) when the e2e can run.
fn bridge_available() -> Result<(), String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    if !manifest.join("scripts/sany-json.sh").exists() {
        return Err("scripts/sany-json.sh missing".into());
    }
    if !manifest.join("bridge/lib/tla2tools.jar").exists() {
        return Err("tla2tools.jar missing — run scripts/fetch-tla2tools.sh".into());
    }
    if !manifest
        .join("bridge/target/classes/SanyJson.class")
        .exists()
    {
        return Err("bridge classes missing — run scripts/sany-json.sh once".into());
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

#[test]
fn e2e_parses_trivial_spec() {
    if let Err(reason) = bridge_available() {
        eprintln!("skipping e2e (bridge unavailable): {reason}");
        return;
    }

    let doc = tlarc::bridge::parse_spec(&corpus("trivial.tla"))
        .expect("bridge must parse the trivial spec");
    assert_eq!(doc.schema, "tla-ast/v1");
    assert_eq!(doc.module.name, "trivial");
    assert_eq!(doc.module.variables.len(), 1);
    assert_eq!(doc.module.operators.len(), 3);
}

#[test]
fn e2e_verdict_shape_via_check_spec() {
    if let Err(reason) = bridge_available() {
        eprintln!("skipping e2e (bridge unavailable): {reason}");
        return;
    }

    let verdict = tlarc::check::check_spec(&corpus("trivial.tla"), &corpus("trivial.cfg"));
    assert_eq!(verdict.status, "ok");
    assert_eq!(verdict.schema.as_deref(), Some("tla-ast/v1"));
    assert_eq!(verdict.module.as_deref(), Some("trivial"));
    let counts = verdict.counts.expect("counts on ok verdict");
    assert_eq!(counts.operators, 3);
    assert_eq!(counts.variables, 1);
    assert_eq!(counts.constants, 0);
    assert!(verdict.error.is_none());
}

#[test]
fn e2e_semantic_error_is_reported() {
    if let Err(reason) = bridge_available() {
        eprintln!("skipping e2e (bridge unavailable): {reason}");
        return;
    }

    // A spec SANY rejects semantically: '+' is not defined without
    // EXTENDS Naturals (deliberate: the error must surface, not crash).
    let bad = std::env::temp_dir().join("tlarc-bad-spec.tla");
    std::fs::write(
        &bad,
        "---- MODULE bad ----\nVARIABLE x\nInit == x = x + 1\n====\n",
    )
    .expect("write temp spec");

    let verdict = tlarc::check::check_spec(&bad, &corpus("trivial.cfg"));
    assert_eq!(verdict.status, "parse_error");
    let error = verdict.error.expect("error on parse_error verdict");
    assert!(
        error.contains("SANY reported errors") || error.contains("bridge exited"),
        "unexpected error text: {error}"
    );
    let _ = std::fs::remove_file(&bad);
}
