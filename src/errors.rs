pub type MyResult<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unauthorized channel")]
    Auth,
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("{0}")]
    Sumup(#[from] sumup::Error),
    #[error("{0}")]
    Telegram(#[from] teloxide::RequestError),
}
