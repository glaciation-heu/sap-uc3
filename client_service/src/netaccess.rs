use poem_openapi::types::ToJSON;
use reqwest::Client;
use crate::error::{Error, Result};
use mockall::predicate::*;
use mockall::*;

/// Trait used for net access.
#[automock]
pub trait NetAccess {
    /// Execute get request on url.
    async fn get(&self, url: &str) -> Result<Vec<u8>>;
    async fn post(&self, url: &str, body: String) -> Result<()>;
}

/// Wrapper for net access using requests.
pub struct RequestsClient {}
impl NetAccess for RequestsClient {
    async fn get(&self, url: &str) -> Result<Vec<u8>> {
        let resp = Client::new()
            .get(url)
            .header("accepts", "application/json")
            .send().await?;

        if resp.status().as_u16() > 202 {
            let status_code = resp.status().as_u16();
            let result = resp.bytes().await?.to_vec();
            return Err(Error::HttpError { code: status_code, message: result.to_json_string() });
        }
        let result = resp.bytes().await?.to_vec();
        Ok(result)
    }
    async fn post(&self,url: &str,body: String) -> Result<()> {
        let res = Client::new()
            .post(url)
            .body(body)
            .header("accepts", "application/json")
            .header("Content-Type", "application/json")
            .send().await?;
        let is_success = res.status().is_success();
        let body = res.bytes().await?.to_vec();
        let s = String::from_utf8_lossy(&body);
        if is_success {
            Ok(())
        } else {
            Err(Error::from(format!("Unable to register secrets to coordination service {}", s)))
        }
    }
}

impl RequestsClient {
    pub fn new() -> RequestsClient {
        RequestsClient{}
    }
}