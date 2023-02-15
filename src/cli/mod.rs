mod run;

use std::{io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use snafu::ResultExt;

use crate::{consts, error::Result};

#[derive(Debug, Parser)]
#[clap(
    about,
    author,
    version,
    name = "simple-youtube-comment-extractor",
    help_template(consts::HELP_TEMPLATE)
)]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self { Self::parse() }
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Print version info and exit")]
    Version,

    #[command(
        about = "Generate shell completion scripts for bash, zsh, fish, elvish or powershell"
    )]
    Completion { shell: Shell },

    #[command(about = "Extract comments of Youtube videos and export")]
    Run(self::run::Command),
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.commands {
            Commands::Version => {
                let mut stdout = std::io::stdout();
                stdout
                    .write_all(Self::command().render_version().as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Commands::Completion { shell } => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            Commands::Run(command) => command.run(),
        }
    }
}

#[allow(dead_code)]
fn init_tracing() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir_path = PathBuf::from(consts::LOG_PATH);
    std::fs::create_dir_all(&log_dir_path).context(crate::error::CreateLogDirectorySnafu)?;

    let appender = tracing_appender::rolling::daily(&log_dir_path, "cli.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_appender)
        .with_ansi(true)
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};

    #[test]
    fn test_command_simple() {
        match Cli::parse_from(["program_name", "version"]).commands {
            Commands::Version => (),
            _ => panic!(),
        }
    }
}
