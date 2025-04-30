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

    // -- notify

    // -- result

    // -- secrets

    // -- Externals
    #[error("io error {0}")]
    Io(#[from] std::io::Error), // as example

    #[error("json error {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("net error {0}")]
    ReqwestError(#[from] reqwest::Error)

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
        match self {
            Error::CollaborationNotFound { collab_id: _ } => StatusCode::NOT_FOUND,
            Error::Unprocessable{message: _} => StatusCode::UNPROCESSABLE_ENTITY,
            Error::HttpError { code, message: _ } => StatusCode::from_u16(*code).unwrap(),
            Error::ReqwestError(err) => {
                match err.status() {
                    Some(status) => status,
                    None => StatusCode::INTERNAL_SERVER_ERROR
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
        event!(Level::INFO, "Return error response: {:?}", body);
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
                }
            ]
        }
    }

    fn register(registry: &mut Registry) {
        let _ = registry;
    }
}
// endregion: --- Error Boilerplate
