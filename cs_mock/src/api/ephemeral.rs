use std::env;

use cs_interface::{CarbynestackConfig, CarbynestackProvider, CsClient};
use poem::Result;
use poem_openapi::{param::{Path, Query}, payload::Json, ApiResponse, Object, OpenApi};
use tracing::{event, Level};

use num_integer::Integer;

use crate::{api::amphora::{delete_secret, get_secrets_internal, P}, computation::run_computation};
fn cs_config() -> CarbynestackConfig {
    CarbynestackConfig {
        prime: "198766463529478683931867765928436695041".to_string(),
        r: "141515903391459779531506841503331516415".to_string(),
        rinv: "133854242216446749056083838363708373830".to_string(),
        no_ssl_validation: true,
        providers: vec![
            CarbynestackProvider {
                id: 1,
                amphora_service_url: "http://127.0.0.1.sslip.io/0/amphora".to_string(),
                castor_service_url: "http://127.0.0.1.sslip.io/0/castor".to_string(),
                ephemeral_service_url: "http://127.0.0.1.sslip.io/0".to_string(),
                base_url:  "http://127.0.0.1.sslip.io/0".to_string(),
            },
            CarbynestackProvider {
                id: 2,
                amphora_service_url: "http://127.0.0.1.sslip.io/1/amphora".to_string(),
                castor_service_url: "http://127.0.0.1.sslip.io/1/castor".to_string(),
                ephemeral_service_url: "http://127.0.0.1.sslip.io/1".to_string(),
                base_url:  "http://127.0.0.1.sslip.io/1".to_string(),
            }
        ],
    }
}

fn use_mpc() -> bool {
    // Check if USE_MPSPDZ environment variable is set.
    env::var("USE_MPSPDZ").is_ok()
}

const RESULT_UUID:&str = "00000000-0000-0000-0000-000000000000";
pub struct EphemeralApi;
#[OpenApi]
impl EphemeralApi {
/// Trigger MPC function
#[oai(path="/:vcp_id/", method="post")]
    async fn execute_vcp(&self,
        #[oai(name= "vcp_id")]
        vcp_id: Path<i32>, data: Json<StartComputationPayload>, compile: Query<bool>) -> Result<ExecuteResp> {
        
        event!(Level::INFO, "request for execution from vcp {}", vcp_id.0);
        let resp_obj = ComputationResponse {
            response: vec![RESULT_UUID.to_string()]
        };
        if vcp_id.0 == 0 && use_mpc() {
            // delete prev results
            delete_secret(&RESULT_UUID.to_string());

            event!(Level::INFO, "Request to run computation with data {:?}", data.amphora_params);
            let mut secret_data = Vec::new();
            for id in &data.amphora_params {
                let shares = get_secrets_internal(id);
                for share in shares {
                    secret_data.push(share.0.checked_add(&share.1).unwrap().mod_floor(&P));
                }
            }
            let res = run_computation(data.code.clone(), secret_data).await?;
            let secrets = res.into_iter().map(|s| s.to_string()).collect();
            let _ = tokio::spawn(async move {
                let client = cs_interface::JavaCsClient::new(cs_config()).expect("Unable to create Java CS client");
                client.create_secrets(secrets, Some(RESULT_UUID.to_string())).expect("Error creating secret!");
            }).await;
        } else {
            event!(Level::INFO, "Do nothing");
        }
        Ok(ExecuteResp::OK(Json(resp_obj)))
    }
}

#[derive(Object)]
#[oai(rename_all="camelCase")]
struct OutputOptions {
    #[oai(rename="type")]
    output_type: String,
}

#[derive(Object)]
#[oai(rename_all="camelCase")]
struct StartComputationPayload {
    /// ID of the computation process. This is used to synchronize and link the computation processes of the VC.
    game_id: String,
    /// A list of Amphora SecretShare IDs which should be used as input to the computation.
    amphora_params: Vec<String>,
    secret_params: Vec<String>,
    /// Defines how and where the result of the computation is stored. Although multiple options are available, this must be set to 'AMPHORASECRET'.
    output: OutputOptions,
    /// MPC function code to be compiled and run for this computation process.
    code: String
}

#[derive(Object)]
#[oai(rename_all="camelCase")]
struct ComputationResponse {
    response: Vec<String>
}

#[derive(ApiResponse)]
enum ExecuteResp {
    #[oai(status = 200)]
    OK(Json<ComputationResponse>)
}
