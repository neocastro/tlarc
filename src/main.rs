use clap::Parser;

use tlarc::VERSION;

/// A Rust reimplementation of the TLC model checker for TLA+.
#[derive(Parser, Debug)]
#[command(name = "tlarc", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Parse a spec through the SANY bridge (checking not implemented yet).
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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { spec, config, json } => {
            // M0.5: end-to-end parse via the SANY bridge; model checking
            // lands in M1.
            let msg = format!(
                "tlarc {VERSION}: parsed spec '{spec}' with config '{config}'; \
                 checking not implemented yet (M0 bootstrap)"
            );
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "version": VERSION,
                        "status": "not_implemented",
                        "spec": spec,
                        "config": config,
                        "message": msg,
                    })
                );
            } else {
                println!("{msg}");
            }
        }
    }
}
