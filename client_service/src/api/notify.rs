use crate::error::{Error, Result};
use poem_openapi::{ApiResponse, Object};
use serde::Deserialize;
use tracing::{event, Level};

#[derive(Object, Deserialize)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct NotifyBody {
    pub message: String,
    pub code: i32,
    pub collaboration_id: i32,
    pub secret_id: String
}

#[derive(ApiResponse, PartialEq, Debug)]
pub enum NotifyResponse {
    /// Notification accepted
    #[oai(status = 202)]
    NotificationAccepted,
}

pub fn notify(notification: NotifyBody) -> Result<NotifyResponse> {
    if notification.code != 200 {
        return Err(Error::Unprocessable { message: "Not waiting for a notification".to_string() });
    }
    event!(Level::INFO, "A result notification was received for collaboration {}.", notification.collaboration_id);
    Ok(NotifyResponse::NotificationAccepted)
}

#[cfg(test)]
mod notify_tests {
    use super::*;

    #[test]
    fn test_notify() {
        let res = notify(NotifyBody{
            message: "Hello World!".to_string(),
            code: 200,
            collaboration_id: 0,
            secret_id: "-".to_string()
        });
        if let Ok(resp) = res {
            assert_eq!(resp, NotifyResponse::NotificationAccepted);
        }
    }

    #[test]
    fn test_notify_failing() {
        let res = notify(NotifyBody{
            message: "Hello World!".to_string(),
            code: 400,
            collaboration_id: 0,
            secret_id: "-".to_string()
        });
        if let Err(Error::Unprocessable { message: _}) = res {
            assert!(true);
        } else {
            assert!(false, "Expected unprocessable response");
        }
    }
}
