use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Command { source: Box<dyn CommandError> },

    #[snafu(display("Error occurs while initialising Tokio runtime.\nError: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not create directory for saving log.\nError: {source}"))]
    CreateLogDirectory { source: std::io::Error },
}

impl Error {
    /// Returns exit code constants intended to be passed to
    /// `std::process::exit()`
    pub fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Command { source } => source.exit_code(),
            Self::InitializeTokioRuntime { .. } => exitcode::SOFTWARE,
            Self::CreateLogDirectory { .. } => exitcode::IOERR,
        }
    }
}

pub trait CommandError: snafu::AsErrorSource + snafu::Error + Send + Sync {
    fn exit_code(&self) -> exitcode::ExitCode;
}

impl<T: CommandError + 'static> From<T> for Error {
    fn from(source: T) -> Self { Self::Command { source: Box::new(source) } }
}
