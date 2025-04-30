use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::schema::{participations, collaborations, computation_results, csconfig, csprovider};
use diesel::prelude::*;

#[derive(Insertable, Object)]
#[diesel(table_name = csconfig)]
pub struct NewCsConfig {
    /// The Prime as used by the MPC backend
    pub prime: String,
    /// The auxiliary modulus R as used by the MPC backend
    pub r: String,
    /// The multiplicative inverse for the auxiliary modulus R as used by the MPC backend
    pub rinv: String,
    pub no_ssl_validation: bool
}


#[derive(Queryable, Selectable, AsChangeset, Object, Serialize)]
#[diesel(table_name = csconfig)]
pub struct CsConfig {
    pub id: i32,
    /// The Prime as used by the MPC backend
    pub prime: String,
    /// The auxiliary modulus R as used by the MPC backend
    pub r: String,
    /// The multiplicative inverse for the auxiliary modulus R as used by the MPC backend
    pub rinv: String,
    pub no_ssl_validation: bool
}

#[derive(Insertable, Object, Queryable, Selectable, AsChangeset, Serialize)]
#[diesel(table_name = csprovider)]
pub struct CsProvider {
    pub config_id: i32,
    pub id: i32,
    pub amphora_service_url: String,
    pub castor_service_url: String,
    pub ephemeral_service_url: String,
    pub base_url: String,
}

#[derive(Insertable, Object)]
#[diesel(table_name = collaborations)]
pub struct NewCollaboration {
    /// Name of the collaboration
    pub name: String,
    /// SPDZ Programm (base64 encoded)
    pub mpc_program: String,
    /// input parameter cs specification (base64 encoded)
    pub csv_specification: String,
    // Number of participating parties
    pub participation_number: i32,
    // id of cs configuration
    pub config_id: i32,
    // endpoints of output parties
    pub output_parties: Option<Vec<Option<String>>>
}

#[derive(Queryable, Selectable, AsChangeset, Object)]
#[diesel(table_name = collaborations)]
pub struct Collaboration {
    pub id: i32,
    /// Name of the collaboration
    pub name: String,
    /// SPDZ Programm (base64 encoded)
    pub mpc_program: String,
    /// input parameter cs specification (base64 encoded)
    pub csv_specification: String,
    // Number of participating parties
    pub participation_number: i32,
    // id of cs configuration
    pub config_id: i32,
    // endpoints of output parties
    pub output_parties: Option<Vec<Option<String>>>
}

#[derive(Insertable)]
#[diesel(table_name = participations)]
#[diesel(primary_key(user_id, collaboration_id))]
pub struct NewParticipation {
    pub collaboration_id: i32,
    pub party_id: i32
}

#[derive(Queryable, Selectable, AsChangeset, Object, Serialize, Deserialize)]
#[diesel(table_name = participations)]
#[diesel(primary_key(collaboration_id, party_id))]
#[diesel(belongs_to(Collaboration, foreign_key = collaboration_id))]
pub struct Participation {
    pub collaboration_id: i32,
    pub party_id: i32,
    pub secret_ids: Option<Vec<Option<String>>>,
}

#[derive(Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = computation_results)]
#[diesel(primary_key(collab_id))]
#[diesel(belongs_to(Collaboration, foreign_key = collab_id))]
pub struct ComputationResult {
    pub collab_id: i32,
    pub result_ids: Option<Vec<Option<String>>>,
    pub finished: bool,
    pub error: Option<String>
}
