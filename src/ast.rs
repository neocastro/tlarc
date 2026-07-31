//! The tlarc AST: Rust-side mirror of the JSON emitted by the `sany-json`
//! bridge (see `docs/ast-schema.md` for the contract, `bridge/src/SanyJson.java`
//! for the producer).

use serde::{Deserialize, Serialize};

/// The AST schema version. Must match `SanyJson.SCHEMA` (bridge) and
/// `docs/ast-schema.md`. Bump all three together.
pub const AST_SCHEMA: &str = "tla-ast/v1";

/// Root of the bridge output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub schema: String,
    pub module: Module,
}

/// A resolved TLA+ module (the root module of the spec).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub constants: Vec<OpDecl>,
    pub variables: Vec<OpDecl>,
    pub operators: Vec<OpDef>,
}

/// A variable or constant declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpDecl {
    pub name: String,
    /// SANY's printable kind name, e.g. "OpDeclKind" (see ASTConstants.kinds).
    pub kind: String,
}

/// An operator definition: `Name(p1, ..., pn) == body`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpDef {
    pub name: String,
    pub arity: usize,
    pub params: Vec<String>,
    pub body: Expr,
}

/// An expression node. Tagged by the `kind` field.
///
/// Unknown constructs are surfaced as [`Expr::Unhandled`] rather than
/// rejected, so the bridge never crashes on features the AST does not yet
/// model — they show up explicitly in the JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Expr {
    /// Operator application: `f(a1, ..., an)`. `operator` is the SANY
    /// SymbolNode name, e.g. `"="` for equality (see docs/ast-schema.md).
    #[serde(rename = "opappl")]
    OpAppl { operator: String, args: Vec<Expr> },
    /// An integer literal. Stored as a decimal string: TLA+ integers are
    /// unbounded, so the value layer decides the representation.
    #[serde(rename = "numeral")]
    Numeral { value: String },
    /// A string literal.
    #[serde(rename = "string")]
    String { value: String },
    /// `LET d1 == e1 ... dn == en IN body`
    #[serde(rename = "letin")]
    LetIn {
        defs: Vec<OpDef>,
        body: Box<Expr>,
    },
    /// A construct the AST does not yet model; `type` names the SANY node
    /// class (or "oparg" for operator arguments).
    #[serde(rename = "unhandled")]
    Unhandled {
        #[serde(rename = "type")]
        r#type: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal document following the schema. Kept in sync with the shim's
    /// real output; the integration test in `bridge/` checks live output.
    #[test]
    fn deserializes_a_minimal_document() {
        let json = r#"{
            "schema": "tla-ast/v1",
            "module": {
                "name": "trivial",
                "constants": [],
                "variables": [{"name": "x", "kind": "VariableDeclKind"}],
                "operators": [
                    {
                        "name": "Init",
                        "arity": 0,
                        "params": [],
                        "body": {
                            "kind": "opappl",
                            "operator": "=",
                            "args": [
                                {"kind": "opappl", "operator": "x", "args": []},
                                {"kind": "numeral", "value": "0"}
                            ]
                        }
                    }
                ]
            }
        }"#;

        let doc: Document = serde_json::from_str(json).expect("fixture must parse");
        assert_eq!(doc.schema, AST_SCHEMA);
        assert_eq!(doc.module.name, "trivial");
        assert_eq!(doc.module.variables.len(), 1);
        assert_eq!(doc.module.variables[0].name, "x");
        assert_eq!(doc.module.operators.len(), 1);
        let body = &doc.module.operators[0].body;
        match body {
            Expr::OpAppl { operator, args } => {
                assert_eq!(operator, "=");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected OpAppl, got {other:?}"),
        }
    }

    #[test]
    fn unknown_fields_are_ignored() {
        // The shim may add fields before the Rust side learns about them;
        // serde must not reject the document.
        let json = r#"{"schema":"tla-ast/v1","module":{"name":"m","constants":[],"variables":[],"operators":[],"future_field":42}}"#;
        let doc: Document = serde_json::from_str(json).expect("unknown fields must be ignored");
        assert_eq!(doc.module.name, "m");
    }
}
