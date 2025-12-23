use derive_more::From;
use poem::{error::ResponseError, http::StatusCode, IntoResponse};
use poem_openapi::{ApiResponse, registry::Registry, registry::{MetaResponses, MetaResponse}};
use tracing::{event, Level};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("custom error {0}")]
    Custom(String),


    #[error("{message}")]
    Unprocessable{message: String},

    #[error("HTTP-Error: status {code}, message: {message}")]
    HttpError{code: u16, message: String},

    // -- CS-Client errors
    #[error("Collaboration with id {collab_id} not found")]
    CollaborationNotFound{collab_id: i32},

    #[error("Internal Server Error {message}")]
    InternalServerError{message: String},

    // -- Externals
    #[error("io error {0}")]
    Io(#[from] std::io::Error), // as example

    #[error("json error {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("net error {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Environment variable error {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("parse from utf8 error {0}")]
    StdError(#[from] std::string::FromUtf8Error),

}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

impl From<String> for Error {
    fn from(val: String) -> Self {
        Self::Custom(val)
    }
}


impl From<cs_interface::Error> for Error {
    fn from(value: cs_interface::Error) -> Self {
        match value {
            cs_interface::Error::EnvVarError(e) => Self::EnvVarError(e),
            cs_interface::Error::CommandError(e) => Self::InternalServerError { message: e },
            cs_interface::Error::Io(error) => Self::Io(error),
            cs_interface::Error::SerdeJson(error) => Self::SerdeJson(error),
            cs_interface::Error::ReqwestError(error) => Self::ReqwestError(error),
            cs_interface::Error::HttpError { code, message } => Self::HttpError{ code, message},
            cs_interface::Error::StdError(e) => Self::StdError(e),
            cs_interface::Error::B64DecodeError(decode_error) => Self::Custom(format!("Decode error: {}", decode_error.to_string())),
        }
    }
}

/// Response error implementation for
impl ResponseError for Error {
    fn status(&self) -> StatusCode {
        match self {
            Error::CollaborationNotFound { collab_id: _ } => StatusCode::NOT_FOUND,
            Error::Unprocessable{message: _} => StatusCode::UNPROCESSABLE_ENTITY,
            Error::HttpError { code, message: _ } => StatusCode::from_u16(*code).unwrap(),
            Error::ReqwestError(err) => {
                match err.status() {
                    Some(status) => status,
                    None => StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    fn as_response(&self) -> poem::Response {
        let body = poem::Body::from_json(serde_json::json!({
            "code": self.status().as_u16(),
            "message": self.to_string(),
        })).unwrap();
        event!(Level::INFO, "Returning {} response", self.status());
        poem::Response::builder().status(self.status()).body(body).into_response()
    }
}

impl ApiResponse for Error {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![
                MetaResponse {
                    description: "Internal Server error",
                    status: Some(500),
                    content: vec![],
                    headers: vec![],
                    status_range: None
                },
                MetaResponse {
                    description: "Not found",
                    status: Some(404),
                    content: vec![],
                    headers: vec![],
                    status_range: None
                },
                MetaResponse {
                    description: "Not finished",
                    status: Some(409),
                    content: vec![],
                    headers: vec![],
                    status_range: None
                },
                MetaResponse {
                    description: "Unprocessable",
                    status: Some(422),
                    content: vec![],
                    headers: vec![],
                    status_range: None
                }
            ]
        }
    }

    fn register(registry: &mut Registry) {
        let _ = registry;
    }
}
// endregion: --- Error Boilerplate
