pub type MyResult<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized channel")]
    Auth,
    #[error("{0}")]
    Env(#[from] envir::Error),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    Sumup(String),
    #[error("{0}")]
    Telegram(#[from] teloxide::RequestError),
}

impl<E: std::fmt::Debug> From<sumup::SdkError<E>> for Error {
    fn from(value: sumup::SdkError<E>) -> Self {
        Self::Sumup(value.to_string())
    }
}
