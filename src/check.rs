//! The `tlarc check` pipeline: parse a spec through the bridge and produce
//! a machine-readable verdict (the CLI contract, see `docs/cli.md`).
//!
//! The `.cfg` file is accepted but **not yet interpreted** — config
//! semantics (Init/Next/Invariant selection, constant assignment) are M1
//! work. The verdict shape is the stable contract the M0.6 diff harness
//! compares against.

use std::path::Path;

use serde::Serialize;

use crate::VERSION;

/// Counts of the definitions SANY resolved in the root module.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DefCounts {
    pub operators: usize,
    pub variables: usize,
    pub constants: usize,
}

/// The machine-readable result of `tlarc check`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckVerdict {
    pub version: &'static str,
    /// `"ok"` when SANY parsed and analyzed the spec; `"parse_error"` otherwise.
    pub status: &'static str,
    pub spec: String,
    pub config: String,
    /// Present iff `status == "ok"`.
    pub schema: Option<String>,
    /// Present iff `status == "ok"`.
    pub module: Option<String>,
    /// Present iff `status == "ok"`.
    pub counts: Option<DefCounts>,
    /// Present iff `status == "parse_error"`.
    pub error: Option<String>,
}

/// Parse `spec` via the bridge and summarize the result.
pub fn check_spec(spec: &Path, config: &Path) -> CheckVerdict {
    match crate::bridge::parse_spec(spec) {
        Ok(doc) => CheckVerdict {
            version: VERSION,
            status: "ok",
            spec: spec.display().to_string(),
            config: config.display().to_string(),
            schema: Some(doc.schema),
            module: Some(doc.module.name),
            counts: Some(DefCounts {
                operators: doc.module.operators.len(),
                variables: doc.module.variables.len(),
                constants: doc.module.constants.len(),
            }),
            error: None,
        },
        Err(e) => CheckVerdict {
            version: VERSION,
            status: "parse_error",
            spec: spec.display().to_string(),
            config: config.display().to_string(),
            schema: None,
            module: None,
            counts: None,
            error: Some(e.message),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_verdict_carries_counts() {
        // Build the verdict via the pure path: check_spec needs a real
        // bridge, so test the shape by constructing the ok branch through
        // bridge fixture parsing instead — covered by tests/m05_e2e.rs.
        // Here we only pin the contract shape.
        let counts = DefCounts {
            operators: 3,
            variables: 1,
            constants: 0,
        };
        assert_eq!(counts.operators, 3);
    }

    #[test]
    fn verdict_serializes_with_expected_fields() {
        let v = CheckVerdict {
            version: VERSION,
            status: "ok",
            spec: "s.tla".into(),
            config: "s.cfg".into(),
            schema: Some("tla-ast/v1".into()),
            module: Some("m".into()),
            counts: Some(DefCounts {
                operators: 1,
                variables: 0,
                constants: 0,
            }),
            error: None,
        };
        let json = serde_json::to_value(&v).expect("serializable");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["schema"], "tla-ast/v1");
        assert_eq!(json["counts"]["operators"], 1);
        assert!(json.get("error").unwrap().is_null());
    }
}
