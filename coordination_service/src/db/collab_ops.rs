use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::error::{Error, Result};

use crate::db::{establish_connection, models::{Collaboration, ComputationResult, NewCollaboration}};

/// Create new participation between user and collaboration
pub fn create(collaboration: NewCollaboration, db_url: &str) -> Result<Collaboration> {
    use crate::schema::collaborations;

    let mut connection = establish_connection(db_url)?;
    let collab = diesel::insert_into(collaborations::table)
        .values(&collaboration)
        .get_result(&mut connection)?;
    Ok(collab)
}

pub fn list(db_url: &str) -> Result<Vec<Collaboration>> {
    use crate::schema::collaborations::dsl::*;
    let mut connection = establish_connection(db_url)?;

    let collaboration_list = collaborations.load::<Collaboration>(&mut connection)?;
    Ok(collaboration_list)
}

pub fn get(collab_id: i32, db_url: &str) -> Result<Collaboration> {
    use crate::schema::collaborations;

    let mut connection = establish_connection(db_url)?;
    let collab = collaborations::dsl::collaborations.find(collab_id)
        .first(&mut connection)?;
    Ok(collab)
}

pub fn delete(collab_id: i32, db_url: &str) -> Result<usize> {
    use crate::schema::collaborations::dsl::*;
    use crate::schema::computation_results;
    let mut connection = establish_connection(db_url)?;
    let _ = diesel::delete(computation_results::dsl::computation_results.find(collab_id))
        .execute(&mut connection);
    let removed_cound = diesel::delete(collaborations.find(collab_id)).execute(&mut connection)?;
    if removed_cound < 1 {
        Err(crate::error::Error::CollaborationNotFound { collab_id })
    } else {
        Ok(removed_cound)
    }
}

pub fn result_ids(collab_id: i32, db_url: &str) -> Result<Vec<String>> {
    use crate::schema::computation_results;
    let mut connection = establish_connection(db_url)?;

    let results = computation_results::dsl::computation_results.find(collab_id)
        .get_result::<ComputationResult>(&mut connection)?;
    if let Some(ids) = results.result_ids {
        return Ok(ids.into_iter().filter_map(|e| e).collect::<Vec<String>>());
    } else if let Some(error_message) = results.error {
        return Err(Error::MPCExecutionFailed(error_message));
    }
    return Err(Error::ProcessingNotFinished);
}

pub fn add_started_result(collab_id: i32, db_url: &str) -> Result<()> {
    use crate::schema::computation_results;
    let mut connection = establish_connection(db_url)?;
    let comp_result = ComputationResult {
        collab_id,
        result_ids: None,
        finished: false,
        error: None
    };
    let _ = diesel::insert_into(computation_results::table)
        .values(&comp_result)
        .execute(&mut connection)?;
    Ok(())
}

pub fn set_result_failed(id_of_collaboration: i32, message: String, db_url: &str) -> Result<()> {
    use crate::schema::computation_results::dsl::*;
    let mut connection = establish_connection(db_url)?;

    diesel::update(computation_results.find(id_of_collaboration))
        .set((finished.eq(true), error.eq(Some(message))))
        .execute(&mut connection)?;
    Ok(())
}

pub fn set_result_finished(id_of_collaboration: i32, ids_of_result: Vec<Option<String>>, db_url: &str) -> Result<()> {
    use crate::schema::computation_results::dsl::*;
    let mut connection = establish_connection(db_url)?;

    diesel::update(computation_results.find(id_of_collaboration))
        .set((finished.eq(true), result_ids.eq(Some(ids_of_result))))
        .execute(&mut connection)?;
    Ok(())
}

pub fn add_output_party(collab_id: i32, _party_id: i32, party_client_endpoint: String, db_url: &str) -> Result<()> {
    use crate::schema::collaborations::dsl::*;

    let mut connection = establish_connection(db_url)?;
    let old_array: Collaboration = collaborations
        .find(collab_id)
        .get_result::<Collaboration>(&mut connection)?;
    let output = match old_array.output_parties {
        Some(mut parties) => {
            parties.push(Some(party_client_endpoint));
            parties
        },
        None => vec![Some(party_client_endpoint)],
    };
    diesel::update(collaborations.find(collab_id))
        .set(output_parties.eq(Some(output)))
        .execute(&mut connection)?;
    Ok(())
}