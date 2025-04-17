use core::time;
use std::thread;

use poem::Result;
use poem_openapi::{param::{Path, Query}, payload::Json, ApiResponse, Object, OpenApi};
use tracing::{event, Level};

const RESULT_UUID:&str = "00000000-0000-0000-0000-000000000000";
pub struct EphemeralApi;
#[OpenApi(prefix_path="/")]
impl EphemeralApi {

    /// Trigger MPC function
    #[oai(path="/:vcp_id/", method="post")]
    async fn execute_vcp(&self,
        #[oai(name= "vcp_id")]
        vcp_id: Path<i32>, data: Json<StartComputationPayload>, compile: Query<bool>) -> Result<ExecuteResp> {
        event!(Level::INFO, "request for execution form vcp {}", vcp_id.0);
        let resp_obj = ComputationResponse {
            response: vec![RESULT_UUID.to_string()]
        };
        // sleep 2 seconds
        thread::sleep(time::Duration::from_secs(2));
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
