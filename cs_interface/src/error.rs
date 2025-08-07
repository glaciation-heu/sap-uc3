pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {

    // custom
    #[error("Error executing command {0}")]
    CommandError(String),

    #[error("io error {0}")]
    Io(#[from] std::io::Error),

    #[error("json error {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("{0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("net error {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("HTTP-Error: status {code}, message: {message}")]
    HttpError{code: u16, message: String},

    #[error("parse from utf8 error {0}")]
    StdError(#[from] std::string::FromUtf8Error),

    #[error("b64 decode error {0}")]
    B64DecodeError(#[from] base64::DecodeError),
}
