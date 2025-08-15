use poem_openapi::{payload::Json , ApiResponse, Object};
use serde::{Serialize, Deserialize};
use tracing::{event, Level};
use crate::{api::config::CarbynestackConfig, db::{collab_ops, models::Participation, participation_ops}, error::Result};

#[derive(Object, Deserialize, Serialize)]
pub struct RegisterParticipationResponseBody {
    cs_config: CarbynestackConfig,
    csv_specification: String
}

/// Response on registering a new participation
#[derive(ApiResponse)]
pub enum RegisterParticipationResponse {
    /// Successfully create participation
    #[oai(status = 200)]
    OK(Json<Participation>),

    /// Already added as participating party
    #[oai(status = 208)]
    AlreadyAdded(Json<RegisterParticipationResponseBody>),
}

pub fn register_input_party(collaboration_id: i32, party_id: i32, db_url: &str) -> Result<RegisterParticipationResponse> {
    let resp = participation_ops::create_participation(collaboration_id, party_id, db_url)?;
    event!(Level::INFO, "Party {} was registered as input-party to the collaboration with ID {}.", party_id, collaboration_id);
    Ok(RegisterParticipationResponse::OK(Json(resp)))
}

/// Response on deleting a collaboration
#[derive(ApiResponse)]
pub enum DeleteParticipationResponse {
    /// Successfully removed from participating parties.
    #[oai(status = 200)]
    Removed,
}

pub fn delete(collaboration_id: i32, party_id: i32, db_url: &str) -> Result<DeleteParticipationResponse> {
    participation_ops::delete_participation(collaboration_id, party_id, db_url)?;
    Ok(DeleteParticipationResponse::Removed)
}

#[derive(ApiResponse)]
pub enum ListParticipationsResponse {
    #[oai(status = 200)]
    OK(Json<Vec<Participation>>),
}

pub fn list(collaboration_id: i32, db_url: &str) -> Result<ListParticipationsResponse> {
    Ok(ListParticipationsResponse::OK(Json(participation_ops::list_participations(collaboration_id, db_url)?)))
}

/// Response on registering parties
#[derive(ApiResponse)]
pub enum RegisterOutputPartyResponse {
    /// Party was registered successfully
    #[oai(status = 200)]
    Ok,
}

pub fn register_output_party(collaboration_id: i32, party_id: i32, party_client_endpoint: String, db_url: &str) -> Result<RegisterOutputPartyResponse> {
    let _ = collab_ops::add_output_party(collaboration_id, party_id, party_client_endpoint, db_url)?;
    event!(Level::INFO, "Party {} was registered as output-party to the collaboration with ID {}.", party_id, collaboration_id);
    Ok(RegisterOutputPartyResponse::Ok)
}


#[derive(ApiResponse)]
pub enum PostRegisterUploadResponse {

    /// upload registered successful
    #[oai(status = 200)]
    OK,

    /// The party already registered its output.
    #[oai(status = 208)]
    AlreadyRegistered,
}

pub fn register_upload(collaboration_id: i32, party_id: i32, secret_ids: Vec<String>, db_url: &str) -> Result<PostRegisterUploadResponse> {
    participation_ops::upload_done(collaboration_id, party_id, secret_ids, db_url)?;
    Ok(PostRegisterUploadResponse::OK)
}
