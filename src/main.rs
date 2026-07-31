use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use tlarc::check::check_spec;

/// A Rust reimplementation of the TLC model checker for TLA+.
#[derive(Parser, Debug)]
#[command(name = "tlarc", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Parse a spec through the SANY bridge and report a verdict.
    ///
    /// The `.cfg` file is accepted but not yet interpreted (M1 work); the
    /// JSON verdict shape is documented in docs/cli.md.
    Check {
        /// The TLA+ specification file.
        spec: String,
        /// The TLC model configuration file.
        config: String,
        /// Emit a machine-readable JSON verdict.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { spec, config, json } => {
            let verdict = check_spec(Path::new(&spec), Path::new(&config));
            let ok = verdict.status == "ok";

            if !ok {
                // Forward the SANY diagnostics to stderr in both output
                // modes; the JSON verdict also carries them (docs/cli.md).
                eprintln!(
                    "tlarc: parse error for '{}': {}",
                    verdict.spec,
                    verdict.error.as_deref().unwrap_or("unknown error")
                );
            }

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&verdict).expect("serializable")
                );
            } else if ok {
                let counts = verdict.counts.as_ref().expect("counts on ok verdict");
                println!(
                    "tlarc {}: parsed module '{}' ({} operators, {} variables, {} constants)",
                    verdict.version,
                    verdict.module.as_deref().unwrap_or("?"),
                    counts.operators,
                    counts.variables,
                    counts.constants
                );
            }

            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}
