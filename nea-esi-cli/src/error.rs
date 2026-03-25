use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Esi(nea_esi::EsiError),
    Config(String),
    Auth(String),
    Io(std::io::Error),
    Output(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Esi(e) => write!(f, "{e}"),
            Self::Config(msg) => write!(f, "Config error: {msg}"),
            Self::Auth(msg) => write!(f, "Auth error: {msg}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Output(msg) => write!(f, "Output error: {msg}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<nea_esi::EsiError> for CliError {
    fn from(e: nea_esi::EsiError) -> Self {
        Self::Esi(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
