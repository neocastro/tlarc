//! tlarc — a Rust reimplementation of the TLC model checker.
//!
//! Library surface for M0 (bootstrap). The CLI (`main.rs`) is the product;
//! the library exists so tests and the diff harness can call the checker
//! without going through argv parsing.

/// The tlarc version, mirrored from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Path to the `tla2tools.jar` used by the Java SANY bridge,
/// relative to the repository root (fetched by `scripts/fetch-tla2tools.sh`).
pub const TLA2TOOLS_JAR_RELPATH: &str = "bridge/lib/tla2tools.jar";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn jar_relpath_is_pinned() {
        assert_eq!(TLA2TOOLS_JAR_RELPATH, "bridge/lib/tla2tools.jar");
    }
}
