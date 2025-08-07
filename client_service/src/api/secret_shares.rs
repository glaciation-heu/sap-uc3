use cs_interface::CsClient;
use poem_openapi::{payload::Json, ApiResponse, Object};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};
use uuid::Uuid;
use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct SecretTag {
    key: String,
    value: String,
    value_type: String
}

#[derive(Serialize, Deserialize, Object)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct SecretShare {
    secret_id: String,
    tags: Vec<SecretTag>,
    data: String,
    secret_shares: String,
    r_shares: String,
    v_shares: String,
    w_shares: String,
    u_shares: String,
}

#[derive(Serialize, Object)]
pub struct GetSecretSharesResult {
    result: Vec<Option<SecretShare>>,
}

#[derive(ApiResponse)]
pub enum GetSecretShareResponse {
    /// Computation ID response
    #[oai(status = 200)]
    OK(Json<GetSecretSharesResult>),
}

pub async fn get_secret_share(secret_id: String, cs_client: &impl CsClient) -> Result<GetSecretShareResponse> {
    //let config = get_config(collab_id, &RequestsClient::new()).await?;

    let comp_parties = cs_client.get_comp_party_urls();
    let req_uuid = Uuid::new_v4();
    let _resp = Client::new()
        .get(format!("{}/amphora/secret-shares/{}?requestId={}", comp_parties[0], secret_id, req_uuid))
        .header("accepts", "application/json")
        .send().await;
    let mut resp_arr: Vec<Option<SecretShare>> = comp_parties.iter().map(|_| None).collect();
    for (i, comp_party) in comp_parties.iter().enumerate().rev() {
        //let comp_party = comp_parties[x];
        let resp_p = Client::new()
            .get(format!("{}/amphora/secret-shares/{}?requestId={}", comp_party, secret_id, req_uuid))
            .header("accepts", "application/json")
            .send().await;
        match resp_p {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.bytes().await?.to_vec();
                let s = String::from_utf8_lossy(&body);
                if status.is_success() {
                    event!(Level::INFO, "{}",s);
                    let data: SecretShare = serde_json::from_str(s.to_string().as_str())?;
                    resp_arr[i] = Some(data);
                } else {
                    event!(Level::ERROR, "Err: {}", s);
                    return Err(Error::InternalServerError { message: s.to_string() })
                }
            },
            Err(e) => {
                event!(Level::ERROR, "Err: {}", e);
                return Err(Error::InternalServerError { message: e.to_string()} );
            }
        }
    }
    let resp = GetSecretSharesResult {
        result: resp_arr
    };
    Ok(GetSecretShareResponse::OK(Json(resp)))
}
