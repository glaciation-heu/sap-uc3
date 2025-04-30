
use std::env;

use derive_more::From;
use poem::{error::ResponseError, http::StatusCode, IntoResponse};
use poem_openapi::{ApiResponse, registry::Registry, registry::{MetaResponses, MetaResponse}};

use tracing::{event, Level};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("custom error {0}")]
    Custom(String),

    #[error("Not waiting")]
    ProcessingNotFinished,

    #[error("{message}")]
    Unprocessable{message: String},

    #[error("MPC execution failed: {0}")]
    MPCExecutionFailed(String),

    #[error("Forbidden!")]
    Forbidden,

    // -- CS-Client errors
    #[error("Collaboration with id {collab_id} not found")]
    CollaborationNotFound{collab_id: i32},

    // -- Externals
    #[error("io error {0}")]
    Io(#[from] std::io::Error), // as example

    #[error("json error {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("net error {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("database error {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("database connection error {0}")]
    DieselConnectionError(#[from] diesel::ConnectionError),

    #[error("b64 decode error {0}")]
    B64DecodeError(#[from] base64::DecodeError),

    #[error("Environment variable error {0}")]
    EnvVarError(#[from] env::VarError)
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

/// Response error implementation for
impl ResponseError for Error {
    fn status(&self) -> StatusCode {
        // TODO check why status is not used in response!
        event!(Level::INFO, "Getting status");
        match self {
            Error::CollaborationNotFound { collab_id: _ } => StatusCode::NOT_FOUND,
            Error::Unprocessable{message: _} => StatusCode::UNPROCESSABLE_ENTITY,
            Error::ProcessingNotFinished => StatusCode::from_u16(409).unwrap(),
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::DieselError(err) => {
                event!(Level::INFO, "Diesel error: {:?}", err);
                println!("Disel error: {:?}", err);
                match err {
                    diesel::NotFound => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    fn as_response(&self) -> poem::Response {
        let body = poem::Body::from_json(serde_json::json!({
            "code": self.status().as_u16(),
            "message": self.to_string(),
        })).unwrap();
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
                    headers: vec![]
                },
                MetaResponse {
                    description: "Not found",
                    status: Some(404),
                    content: vec![],
                    headers: vec![]
                },
                MetaResponse {
                    description: "Not finished",
                    status: Some(409),
                    content: vec![],
                    headers: vec![]
                },
                MetaResponse {
                    description: "Unprocessable",
                    status: Some(422),
                    content: vec![],
                    headers: vec![]
                },
                MetaResponse {
                    description: "Forbidden",
                    status: Some(403),
                    content: vec![],
                    headers: vec![]
                }
            ]
        }
    }

    fn register(registry: &mut Registry) {
        let _ = registry;
    }
}
