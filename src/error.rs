use std::io;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(serde_yaml::Error),
    InvalidConfig,
    ApiClient(tapo::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app error")
    }
}

impl std::error::Error for Error {}

impl From<tapo::Error> for Error {
    fn from(value: tapo::Error) -> Self {
        Self::ApiClient(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Parse(value)
    }
}