use std::io::Write;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use snafu::ResultExt;

use crate::{consts,error::Result};

#[derive(Debug, Parser)]
#[clap(about, author, version, name = "simple-youtube-comment-extrator", help_template(consts::HELP_TEMPLATE))]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Print version info and exit")]
    Version,

    #[command(
        about = "Generate shell completion scripts for bash, zsh, fish, elvish or powershell"
    )]
    Completion { shell: Shell },
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
        }
    }
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
