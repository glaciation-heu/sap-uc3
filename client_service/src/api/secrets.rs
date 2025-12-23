use cs_interface::{ClearTextSecret, CsClient, NetAccess};
use poem_openapi::{payload::{Json, PlainText}, types::{ToJSON, multipart::Upload}, ApiResponse, Multipart};
use tracing::{event, Level};
use crate::{error::{Error, Result}};

use super::utils;

#[derive(Debug, Multipart)]
pub struct UploadPayload {
    /// secret data as csv
    data_csv: Upload,
    uuid: Option<String>
}

#[derive(ApiResponse)]
pub enum UploadResponse {
    /// Secret created successfully.
    #[oai(status = 200)]
    OK(
        /// The ids of the created secret
        Json<Vec<String>>
    ),
}

#[derive(ApiResponse)]
pub enum GetSecretResponse {
    /// Computation ID response
    #[oai(status = 200)]
    Secret(Json<ClearTextSecret>),
}

/// Function to upload a secret.
pub async fn upload(collab_id: i32, party_id: i32, secrets: UploadPayload, cs_client: &impl CsClient, net: &impl NetAccess) -> Result<UploadResponse> {
    let secret_arr = match secrets.data_csv.into_string().await {
        Ok(v) => v,
        Err(err) => {
            return Err(Error::from(err.to_string()));
        }
    };
    let mut secret_arr = secret_arr.split("\n").into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
    secret_arr.remove(0); // remove header
    let secret_ids = cs_client.create_secrets(secret_arr, secrets.uuid)?;
    event!(Level::INFO, "Secrets for collaboration {} and party {} successfully created on the computation instances", collab_id, party_id);
    register_upload(&secret_ids, collab_id, party_id, net).await?;
    Ok(UploadResponse::OK(Json(secret_ids)))
}

async fn register_upload(secrets: &Vec<String>, collab_id: i32, party_id: i32, net: &impl NetAccess) -> Result<()> {
    let endpoint = format!("{}/collaboration/{}/confirm-upload/{}", utils::coordinator_uri(), collab_id, party_id);
    event!(Level::DEBUG, "Confirming upload to coordinator {}", endpoint);
    net.post(&endpoint, Json(secrets).to_json_string()).await?;
    event!(Level::INFO, "Secret upload registered with the coordination service.");
    Ok(())
}

pub async fn get(secret_id: String, cs_client: &impl CsClient) -> Result<GetSecretResponse> {
    let secret = cs_client.get_secret(&secret_id)?;
    Ok(GetSecretResponse::Secret(Json(secret)))
}

#[derive(ApiResponse)]
pub enum DelSecretResp {

    /// Removing secrets was successful
    #[oai(status = 200)]
    OK(PlainText<String>),
}

pub async fn delete(secret_ids: Vec<String>, cs_client: &impl CsClient) -> Result<DelSecretResp> {
    let output = cs_client.delete_secrets(secret_ids)?;
    Ok(DelSecretResp::OK(PlainText(output)))
}

#[derive(ApiResponse)]
pub enum ListSecretsResponse {
    /// Computation ID response
    #[oai(status = 200)]
    Secrets(Json<Vec<String>>),
}

pub async fn list_secrets(cs_client: &impl CsClient) -> Result<ListSecretsResponse> {
    let secrets = cs_client.list_secrets()?;
    Ok(ListSecretsResponse::Secrets(Json(secrets)))
}

#[cfg(test)]
mod test {

    // TODO testing upload with multipart file

}
