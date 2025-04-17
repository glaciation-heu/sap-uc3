use poem::web::Data;
use poem_openapi::{param::{Path, Query}, payload::{Json, PlainText}, types::multipart::Upload, ApiResponse, Multipart, Object, OpenApi};
use serde::{Serialize, Deserialize};
use base64::prelude::*;
use tracing::{event, Level};
use crate::{cs_definitions, db::{collab_ops, models::{Collaboration, NewCollaboration}}, error::Result};
use super::{config::{self, get_config, CarbynestackConfig}, participation};

pub struct CollabApi;

#[OpenApi(prefix_path = "/collaboration")]
impl CollabApi {
    /// Create Collaboration 
    #[oai(path = "/", method = "post")]
    async fn add_collaboration(&self, payload: RegisterCollaborationPayload, db_url: Data<&String>) -> Result<RegisterCollaborationResponse> {
        post(payload, db_url.0).await
    }

    /// input_party registers participation. Return input-specification and compute-party config on success.
    #[oai(path = "/:collaboration_id/register-input-party/:party_id", method = "post")]
    async fn register_participation(&self, 
        /// identifier of collaboration to register
        collaboration_id: Path<i32>,
        /// Identifier of party that is registering
        party_id: Path<i32>,
        db_url: Data<&String>
    ) -> Result<participation::RegisterParticipationResponse> {
        participation::register_input_party(collaboration_id.0, party_id.0, db_url.0)
    }

    /// output_party registers participation.
    #[oai(path = "/:collaboration_id/register-output-party/:party_id", method = "post")]
    async fn register_output_party(&self, 
        /// identifier of collaboration to register
        collaboration_id: Path<i32>,
        /// Identifier of party that is registering
        party_id: Path<i32>,
        party_client_endpoint: Query<String>,
        db_url: Data<&String>
    ) -> Result<participation::RegisterOutputPartyResponse> {
        participation::register_output_party(collaboration_id.0, party_id.0, party_client_endpoint.0, db_url.0)
    }

    /// input_party unregisteres from participation.
    #[oai(path = "/:collaboration_id/register-input-party/:party_id", method = "delete")]
    async fn unregister_participation(&self, 
        /// identifier of collaboration to unregister
        collaboration_id: Path<i32>,
        /// Identifier of party that is unregistering
        party_id: Path<i32>,
        db_url: Data<&String>
     ) -> Result<participation::DeleteParticipationResponse> {
        participation::delete(collaboration_id.0, party_id.0, db_url.0)
    }

    /// list participations of collaboration
    #[oai(path = "/:collaboration_id/input-parties", method = "get")]
    async fn get_participations(&self, 
        /// identifier of the collaboration
        collaboration_id: Path<i32>,
        db_url: Data<&String>
    ) -> Result<participation::ListParticipationsResponse> {
        participation::list(collaboration_id.0, db_url.0)
    }

    /// input_party confirms upload done.
    #[oai(path = "/:collaboration_id/confirm-upload/:party_id", method = "post")]
    async fn register_upload(&self,
        /// identifier of the collaboration
        collaboration_id: Path<i32>,
        /// Identifier of party
        party_id: Path<i32>,
        /// ids of created secrets
        secret_ids: Json<Vec<String>>,
        db_url: Data<&String>
    ) -> Result<participation::PostRegisterUploadResponse> {
        participation::register_upload(collaboration_id.0, party_id.0, secret_ids.0, db_url.0)
    }


    /// Delete Collaboration 
    #[oai(path = "/:collaboration_id", method = "delete")]
    async fn delete_collaboration(&self, 
        /// identifier of the collaboration
        collaboration_id: Path<i32>,
        db_url: Data<&String>
    ) -> Result<DeleteCollaborationResponse> {
        delete(collaboration_id.0, db_url.0)
    }

    /// List Collaborations
    #[oai(path = "/", method = "get")]
    async fn list_collaborations(&self,
        db_url: Data<&String>
    ) -> Result<ListCollaborationsResponse> {
        list(db_url.0)
    }

    /// Get result of collaboration
    #[oai(path = "/:collaboration_id/result_ids", method = "get")]
    async fn get_result_ids(&self, 
        /// identifier of the collaboration
        collaboration_id: Path<i32>,
        db_url: Data<&String>
    ) -> Result<GetResultIdsResponse> {
        get_result_ids(collaboration_id.0, db_url.0)
    }

    /// Get Computation Party Config
    #[oai(path = "/:collaboration_id/compute_config", method = "get")]
    async fn get_compute_config(&self, 
        /// identifier of the collaboration
        collaboration_id: Path<i32>,
        db_url: Data<&String>
    ) -> Result<GetConfigResponse> {
        let config = get_config(collaboration_id.0, db_url.0);
        let config = match config {
            Ok(c) => c,
            Err(e) => return Ok(GetConfigResponse::InternalServerError(PlainText(e.to_string()))),
        };
        Ok(GetConfigResponse::Ok(Json(config)))
    }
}

/// Response body after a collaboration was successfully registered.
#[derive(Object, Deserialize, Serialize)]
pub struct RegisterCollaborationResponseBody {
    /// The carbynestack configuration of the registered collaboration
    cs_config: cs_definitions::CsConfig,
    /// The csv header line used to specify the csv data
    csv_specification: String
}

/// Payload for registering a new collaboration
#[derive(Debug, Multipart)]
pub struct RegisterCollaborationPayload {
    /// Name of this collaboration
    name: String,
    /// The MPC program that will be executed
    mpc_program: Upload,
    /// CarbyneStack configuration as described in https://carbynestack.io/documentation/getting-started/cli/
    cs_config: Upload,
    /// The header-line of the csv
    csv_header_line: String,
    /// Number of parties. For now the execution is started if all parties register their secrets
    number_of_parties: i32,
}

#[derive(ApiResponse)]
pub enum RegisterCollaborationResponse {
    /// Successfully added to participating parties.
    #[oai(status = 200)]
    OK(Json<Collaboration>),
    //ComputationResult(Json<RegisterCollaborationResponseBody>),

    /// Already added as participating party.
    #[oai(status = 208)]
    AlreadyAdded(Json<RegisterCollaborationResponseBody>),

    /// Did not find a project with this ID.
    #[oai(status = 404)]
    NotFound,

    /// Internal Server Error
    #[oai(status = 500)]
    InternalServerError(PlainText<String>)
}

/// Post new collaboration
pub async fn post(collab: RegisterCollaborationPayload, db_url: &str) -> Result<RegisterCollaborationResponse> {
    let csconfig_str = collab.cs_config.into_string().await;

    let csconfig_str = match csconfig_str {
        Ok(c) => c,
        Err(_) => return Ok(RegisterCollaborationResponse::InternalServerError(PlainText("Computation service config not provided".to_string())))
    };
    let csconfig = CarbynestackConfig::from_json(&csconfig_str);
    let csconfig = match csconfig {
        Ok(c) => c,
        Err(e) => return Ok(RegisterCollaborationResponse::InternalServerError(PlainText(format!("Unable to decode computation-service config\n{}\n\n{}", e.to_string() , csconfig_str))))
    };

    let mpc_program = collab.mpc_program.into_string().await?;
    let db_config = config::add_config(csconfig, db_url)?;
    let new_collab = NewCollaboration {
        name: collab.name,
        mpc_program: BASE64_STANDARD.encode(mpc_program),
        csv_specification: collab.csv_header_line,
        participation_number: collab.number_of_parties,
        output_parties: None,
        config_id: db_config.id,
    };
    let res = collab_ops::create(new_collab, db_url)?;
    event!(Level::INFO, "A new collaboration with ID {} was created.", res.id);
    Ok(RegisterCollaborationResponse::OK(Json(res)))
}

#[derive(ApiResponse)]
pub enum DeleteCollaborationResponse {
    /// Successfully removed from participating parties.
    #[oai(status = 200)]
    Removed,

    /// Did not find a project with this ID.
    #[oai(status = 404)]
    NotFound,
}

pub fn delete(collab_id: i32, db_url: &str) -> Result<DeleteCollaborationResponse> {
    match collab_ops::delete(collab_id, db_url) {
        Ok(_) => Ok(DeleteCollaborationResponse::Removed),
        Err(_) => Ok(DeleteCollaborationResponse::NotFound)
    }
}

#[derive(ApiResponse)]
pub enum ListCollaborationsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<Collaboration>>),

    #[oai(status = 500)]
    InternalServerError
}

pub fn list(db_url: &str) -> Result<ListCollaborationsResponse> {
    let resp = collab_ops::list(db_url)?;
    Ok(ListCollaborationsResponse::Ok(Json(resp)))
}

/// Response for get result-ids
#[derive(ApiResponse)]
pub enum GetResultIdsResponse {
    /// Success, returns array of result ids
    #[oai(status = 200)]
    Ok(Json<Vec<String>>),

    /// A collaboration with this id does not exist
    #[oai(status = 404)]
    CollaborationNotFound,

    /// An internal server error occurred
    #[oai(status = 500)]
    InternalServerError(PlainText<String>)
}

/// Get result ids stored in the database.
pub fn get_result_ids(collab_id: i32, db_url: &str) -> Result<GetResultIdsResponse> {
    let resp = collab_ops::result_ids(collab_id, db_url)?;
    Ok(GetResultIdsResponse::Ok(Json(resp)))
}


/// Response on the get config request
#[derive(ApiResponse)]
pub enum GetConfigResponse {
    /// Returns the carbynestack config
    #[oai(status = 200)]
    Ok(Json<CarbynestackConfig>),

    /// A collaboration with this id does not exist
    #[oai(status = 404)]
    CollaborationNotFound,

    /// An internal server error occurred
    #[oai(status = 500)]
    InternalServerError(PlainText<String>)
}

#[cfg(test)]
mod test {
    // #[tokio(test)]
    // async fn test_post() {
    // }
}