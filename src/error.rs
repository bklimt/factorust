
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

impl std::convert::From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Error::InvalidArgument(err.to_string())
    }
}
