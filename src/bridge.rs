//! The `sany-json` bridge client: spawns the Java bridge as a subprocess
//! and deserializes its JSON into the tlarc AST.
//!
//! Contract: the bridge's stdout carries **only** JSON (schema
//! `tla-ast/v1`, see `docs/ast-schema.md`); diagnostics go to its stderr.
//! This module preserves that purity — stdout is parsed, stderr is carried
//! in the error.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ast;

/// A failure to invoke the bridge or to interpret its output.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeError {
    pub message: String,
}

impl BridgeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Run the sany-json bridge on `spec` and return the parsed AST document.
pub fn parse_spec(spec: &Path) -> Result<ast::Document, BridgeError> {
    let script = bridge_script()?;

    let output = Command::new("bash")
        .arg(&script)
        .arg(spec)
        .output()
        .map_err(|e| BridgeError::new(format!("failed to spawn bridge: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BridgeError::new(format!(
            "bridge exited with {}: {}",
            output.status,
            stderr.trim()
        )));
    }

    parse_bridge_output(&output.stdout)
}

/// Parse the bridge's stdout as a `tla-ast/v1` document.
///
/// Pure and unit-testable without a JVM; the integration test in
/// `tests/m05_e2e.rs` exercises the real subprocess path.
pub fn parse_bridge_output(stdout: &[u8]) -> Result<ast::Document, BridgeError> {
    let text = String::from_utf8_lossy(stdout);
    serde_json::from_str(&text)
        .map_err(|e| BridgeError::new(format!("bridge emitted invalid JSON: {e}")))
}

/// Locate the bridge runner script.
///
/// Resolution order: `TLARC_BRIDGE_SCRIPT` env var, then
/// `$CARGO_MANIFEST_DIR/scripts/sany-json.sh` (dev checkout), then
/// `./scripts/sany-json.sh` relative to the current directory.
pub fn bridge_script() -> Result<PathBuf, BridgeError> {
    if let Ok(path) = std::env::var("TLARC_BRIDGE_SCRIPT") {
        return Ok(PathBuf::from(path));
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("sany-json.sh");
    if manifest.exists() {
        return Ok(manifest);
    }

    let cwd = std::env::current_dir()
        .map_err(|e| BridgeError::new(format!("cannot read current dir: {e}")))?;
    let relative = cwd.join("scripts").join("sany-json.sh");
    if relative.exists() {
        return Ok(relative);
    }

    Err(BridgeError::new(
        "cannot locate scripts/sany-json.sh — run scripts/fetch-tla2tools.sh first, \
         or set TLARC_BRIDGE_SCRIPT to the bridge runner",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real bridge output for corpus/trivial.tla (schema tla-ast/v1).
    const TRIVIAL_DOC: &str = r#"{
        "schema": "tla-ast/v1",
        "module": {
            "name": "trivial",
            "constants": [],
            "variables": [{"name": "x", "kind": "VariableDeclKind"}],
            "operators": [
                {"name": "Init", "arity": 0, "params": [],
                 "body": {"kind": "opappl", "operator": "=", "args": [
                     {"kind": "opappl", "operator": "x", "args": []},
                     {"kind": "numeral", "value": "0"}]}},
                {"name": "Next", "arity": 0, "params": [],
                 "body": {"kind": "opappl", "operator": "=", "args": [
                     {"kind": "opappl", "operator": "'", "args": [
                         {"kind": "opappl", "operator": "x", "args": []}]},
                     {"kind": "opappl", "operator": "$IfThenElse", "args": [
                         {"kind": "opappl", "operator": "=", "args": [
                             {"kind": "opappl", "operator": "x", "args": []},
                             {"kind": "numeral", "value": "0"}]},
                         {"kind": "numeral", "value": "1"},
                         {"kind": "numeral", "value": "0"}]}]}},
                {"name": "Inv", "arity": 0, "params": [],
                 "body": {"kind": "opappl", "operator": "\\in", "args": [
                     {"kind": "opappl", "operator": "x", "args": []},
                     {"kind": "opappl", "operator": "$SetEnumerate", "args": [
                         {"kind": "numeral", "value": "0"},
                         {"kind": "numeral", "value": "1"}]}]}}
            ]
        }
    }"#;

    #[test]
    fn parses_bridge_output() {
        let doc = parse_bridge_output(TRIVIAL_DOC.as_bytes()).expect("valid bridge output");
        assert_eq!(doc.schema, "tla-ast/v1");
        assert_eq!(doc.module.name, "trivial");
        assert_eq!(doc.module.variables.len(), 1);
        assert_eq!(doc.module.operators.len(), 3);
    }

    #[test]
    fn rejects_invalid_json() {
        let err = parse_bridge_output(b"not json").expect_err("must fail");
        assert!(err.message.contains("invalid JSON"));
    }

    #[test]
    fn rejects_empty_stdout() {
        let err = parse_bridge_output(b"").expect_err("must fail");
        assert!(err.message.contains("invalid JSON"));
    }
}
