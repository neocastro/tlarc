//! The tlarc value system: TLA+ values as Rust types.
//!
//! M1 slice (issue #26): booleans, integers, and strings with the TLA+
//! value ordering.
//!
//! TLA+ integers are unbounded; this slice represents them as `i64`.
//! Arbitrary precision lands later (the AST already carries numerals as
//! decimal strings).

use serde::Serialize;
use std::cmp::Ordering;

/// A TLA+ value.
///
/// Serializes as its natural JSON form: `true` / `false` / `42` / `"s"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

/// The canonical form of a value.
///
/// For these three types the value is already canonical; this function
/// exists so the set/record slices (M1.1b/M1.1c) can reuse one pipeline.
pub fn normalize(v: Value) -> Value {
    v
}

/// The TLA+ value ordering (*Specifying Systems* §14.2.2), restricted to
/// this slice: **FALSE < TRUE < integers < strings**. Mixed types compare
/// by that type rank first; same types compare by contents (integers
/// numerically, not by digit string).
pub fn compare(a: &Value, b: &Value) -> Ordering {
    use Value::*;
    match (a, b) {
        (Bool(x), Bool(y)) => x.cmp(y),
        (Int(x), Int(y)) => x.cmp(y),
        (Str(x), Str(y)) => x.cmp(y),
        (Bool(_), _) => Ordering::Less,
        (_, Bool(_)) => Ordering::Greater,
        (Int(_), _) => Ordering::Less,
        (_, Int(_)) => Ordering::Greater,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_integers_numerically() {
        assert_eq!(compare(&Value::Int(1), &Value::Int(2)), Ordering::Less);
        assert_eq!(compare(&Value::Int(10), &Value::Int(2)), Ordering::Greater);
        // Numeric, not digit-string, ordering: "10" < "2" lexicographically.
        assert_eq!(compare(&Value::Int(2), &Value::Int(10)), Ordering::Less);
    }

    #[test]
    fn orders_bools() {
        assert_eq!(
            compare(&Value::Bool(false), &Value::Bool(true)),
            Ordering::Less
        );
        assert_eq!(
            compare(&Value::Bool(true), &Value::Bool(false)),
            Ordering::Greater
        );
        assert_eq!(
            compare(&Value::Bool(true), &Value::Bool(true)),
            Ordering::Equal
        );
    }

    #[test]
    fn orders_by_type_rank() {
        // FALSE < TRUE < integers < strings
        assert_eq!(compare(&Value::Bool(false), &Value::Int(0)), Ordering::Less);
        assert_eq!(compare(&Value::Bool(true), &Value::Int(-5)), Ordering::Less);
        assert_eq!(
            compare(&Value::Int(0), &Value::Str(String::new())),
            Ordering::Less
        );
        assert_eq!(
            compare(&Value::Int(999), &Value::Str(String::new())),
            Ordering::Less
        );
        // And the reverse directions.
        assert_eq!(
            compare(&Value::Str(String::new()), &Value::Bool(true)),
            Ordering::Greater
        );
        assert_eq!(
            compare(&Value::Str(String::from("a")), &Value::Int(1)),
            Ordering::Greater
        );
        assert_eq!(
            compare(&Value::Int(0), &Value::Bool(true)),
            Ordering::Greater
        );
    }

    #[test]
    fn normalize_is_identity_for_these_types() {
        assert_eq!(normalize(Value::Bool(false)), Value::Bool(false));
        assert_eq!(normalize(Value::Bool(true)), Value::Bool(true));
        assert_eq!(normalize(Value::Int(5)), Value::Int(5));
        assert_eq!(
            normalize(Value::Str(String::from("x"))),
            Value::Str(String::from("x"))
        );
    }

    #[test]
    fn compare_matches_partial_eq() {
        for (a, b) in [
            (Value::Bool(false), Value::Bool(false)),
            (Value::Bool(true), Value::Bool(true)),
            (Value::Int(0), Value::Int(0)),
            (Value::Int(-7), Value::Int(-7)),
            (Value::Str(String::from("")), Value::Str(String::from(""))),
            (
                Value::Str(String::from("abc")),
                Value::Str(String::from("abc")),
            ),
        ] {
            assert_eq!(a, b, "fixture must be equal");
            assert_eq!(
                compare(&a, &b),
                Ordering::Equal,
                "equal values compare Equal"
            );
        }
    }

    #[test]
    fn serializes_as_natural_json() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(Value::Bool(true)).unwrap(),
            json!(true)
        );
        assert_eq!(
            serde_json::to_value(Value::Bool(false)).unwrap(),
            json!(false)
        );
        assert_eq!(serde_json::to_value(Value::Int(42)).unwrap(), json!(42));
        assert_eq!(serde_json::to_value(Value::Int(-3)).unwrap(), json!(-3));
        assert_eq!(
            serde_json::to_value(Value::Str(String::from("s"))).unwrap(),
            json!("s")
        );
    }
}
