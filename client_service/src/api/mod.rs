use poem_openapi::{
    param::Path , payload::Json, Object, OpenApi
};
use crate::{cs_client::JavaCsClient, netaccess::RequestsClient, error::Result};
mod secrets;
mod secret_shares;
mod result;
mod notify;
mod utils;
mod sys_status;

pub struct Api;

#[derive(Object)]
struct UploadPayload {
    name: String,
    desc: Option<String>,
}

#[derive(Object)]
struct UploadRes {
    name: String,
    id: Option<u32>
}

#[OpenApi]
impl Api {

    /// Create secrets.
    #[oai(path = "/secrets/:collab_id/:party_id", method = "post")]
    async fn upload(&self, 
        /// identifier of collaboration
        collab_id: Path<i32>,
        /// identifier of party
        party_id: Path<i32>,
        /// csv of secrets
        payload: secrets::UploadPayload) -> Result<secrets::UploadResponse> {
        let net = RequestsClient::new();
        let client = JavaCsClient::new(collab_id.0, &net).await?;
        secrets::upload(collab_id.0, party_id.0, payload, &client, &net).await
    }

    /// get secret by secret ids.
    #[oai(path = "/raw-secrets/:collab_id/:secret_id", method = "get")]
    async fn get_secrets(&self, 
        collab_id: Path<i32>,
        /// identifiers of secrets to get
        secret_id: Path<String>) -> Result<secrets::GetSecretResponse> {
        let net = RequestsClient::new();
        let client = JavaCsClient::new(collab_id.0, &net).await?;
        secrets::get(secret_id.0, &client).await
    }

    /// list secrets
    #[oai(path = "/raw-secrets/:collab_id", method = "get")]
    async fn list_secrets(&self, collab_id: Path<i32>) -> Result<secrets::ListSecretsResponse> {
        let net = RequestsClient::new();
        let client = JavaCsClient::new(collab_id.0, &net).await?;
        secrets::list_secrets(&client).await
    }

    /// delete secrets with id.
    #[oai(path = "/raw-secrets/:collab_id", method = "delete")]
    async fn del_secrets(&self, 
        collab_id: Path<i32>,
        /// identifiers of secrets to remove
        secret_ids: Json<Vec<String>>) -> Result<secrets::DelSecretResp> {
        let net = RequestsClient::new();
        let client = JavaCsClient::new(collab_id.0, &net).await?;
        secrets::delete(secret_ids.0, &client).await
    }

    /// Get computation results (checks if computation is ready).
    #[oai(path = "/result/:collab_id/:party_id", method = "get")]
    async fn get_result(&self, collab_id: Path<i32>, party_id: Path<i32>) -> Result<result::ResultResponse> {
        let net = RequestsClient::new();
        let client = JavaCsClient::new(collab_id.0, &net).await?;
        result::result(collab_id.0, party_id.0, &client, &net).await
    }

    /// notify client that results are finished.
    #[oai(path = "/notify", method = "put")]
    async fn notify(&self, 
        /// identifier of collaboration
        notification: Json<notify::NotifyBody>) -> Result<notify::NotifyResponse> {
        notify::notify(notification.0)
    }

    /// Get secret shares.
    #[oai(path = "/secret_shares/:coord_id/:secret_id", method = "get")]
    async fn get_secret_shares(&self, coord_id: Path<i32>, secret_id: Path<String>) -> Result<secret_shares::GetSecretShareResponse> {
        secret_shares::get_secret_share(secret_id.0, coord_id.0).await
    }

    /// Returns status code 200. Used to check if service is available.
    #[oai(path = "/ping", method = "get")]
    async fn ping(&self) -> Result<()> {
        Ok(())
    }

    /// Get system informations.
    #[oai(path = "/sys_status", method = "get")]
    async fn sys_status(&self) -> Result<sys_status::SysStatusResponse> {
        sys_status::sys_status()
    }
}
