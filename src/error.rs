use std::path::PathBuf;

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Command { source: Box<dyn CommandError> },
}

impl Error {
    /// Returns exit code constants intended to be passed to
    /// `std::process::exit()`
    pub fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Command { source } => source.exit_code()
        }
    }
}

pub trait CommandError: snafu::AsErrorSource + snafu::Error + Send + Sync {
    fn exit_code(&self) -> exitcode::ExitCode;
}

impl<T: CommandError + 'static> From<T> for Error {
    fn from(source: T) -> Self { Self::Command { source: Box::new(source) } }
}
