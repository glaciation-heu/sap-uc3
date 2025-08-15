use cs_interface::{CsClient, JavaCsClient};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use poem_openapi::Object;
use tracing::{event, Level};

use crate::{
    api::config::get_config, db::{
        collab_ops, establish_connection, models::{NewParticipation, Participation}
    }, error::Result, notification_service::notify_parties
};

#[derive(Object)]
#[oai(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub message: String,
    pub code: i32,
    pub collaboration_id: i32,
    pub secret_id: Option<String>
}

/// Create new participation between user and collaboration
pub fn create_participation(collaboration_id: i32, party_id: i32, db_url: &str) -> Result<Participation> {
    use crate::schema::participations;

    let mut connection = establish_connection(db_url)?;

    // try to get collaboration
    let _ = collab_ops::get(collaboration_id, db_url)?;

    let new_participation = NewParticipation {
        collaboration_id,
        party_id,
    };

    let participation = diesel::insert_into(participations::table)
        .values(&new_participation)
        .get_result(&mut connection)?;
    Ok(participation)
}

pub fn list_participations(collab_id: i32, db_url: &str) -> Result<Vec<Participation>> {
    use crate::schema::participations::dsl::*;
    let mut connection = establish_connection(db_url)?;
    let participants = participations
        .filter(collaboration_id.eq(collab_id))
        .get_results::<Participation>(&mut connection)?;
    Ok(participants)
}

pub fn delete_participation(collaboration: i32, party: i32, db_url: &str) -> Result<()> {
    use crate::schema::participations::dsl::*;
    let mut connection = establish_connection(db_url)?;
    diesel::delete(participations.find((collaboration, party))).execute(&mut connection)?;
    Ok(())
}

pub fn upload_done(collaboration: i32, party: i32, ids: Vec<String>, db_url: &str) -> Result<Participation> {
    use crate::schema::participations::dsl::*;
    let mut connection = establish_connection(db_url)?;
    let update_participation = diesel::update(participations.find((collaboration, party)))
        .set(
            secret_ids.eq(Some(
                ids.into_iter()
                    .map(|secret_id| Some(secret_id))
                    .collect::<Vec<Option<String>>>(),
            )),
        )
        .get_result::<Participation>(&mut connection)?;
    let database_string = db_url.to_string();
    tokio::spawn(async move { check_and_execute(collaboration, &database_string).await });
    Ok(update_participation)
}

async fn check_and_execute(collab_id: i32, db_url: &str) -> Result<()> {
    event!(
        Level::INFO,
        "Checking if collaboration {} is ready for execution.",
        collab_id
    );

    let collab = collab_ops::get(collab_id, db_url)?;

    let current_participations = list_participations(collab_id, db_url)?;
    let participation_nr = current_participations.iter().filter(|p| p.secret_ids.is_some()).count();

    if participation_nr < collab.participation_number as usize {
        event!(
            Level::INFO,
            "Not enough participations to start the computation. The current number is {} of {}.",
            participation_nr,
            collab.participation_number as i64
        );
        return Ok(());
    }
    event!(
        Level::INFO,
        "Starting MPC execution of collaboration {}.",
        collab.name
    );

    // Set execution result as started
    collab_ops::add_started_result(collab_id, db_url)?;

    // collect secret ids
    let secret_ids = current_participations
        .into_iter()
        .filter_map(|p| p.secret_ids)
        .flatten()
        .filter_map(|secret_id| secret_id)
        .collect::<Vec<String>>();

    let output_parties = match collab.output_parties {
        Some(parties) => parties
            .into_iter()
            .filter_map(|e| e)
            .collect::<Vec<String>>(),
        None => vec![],
    };
    //let (res, res_ids) =
    //    execute_program(collab.mpc_program, collab.id, secret_ids, collab.config_id, db_url);
    let config = get_config(collab_id, db_url)?;
    let result = JavaCsClient::new(config)?.execute_program(collab.mpc_program, secret_ids);
    let res = match result {
        Ok(res_id) => {
            // write results
            collab_ops::set_result_finished(
                collab_id,
                vec![Some(res_id.clone())], 
                db_url)?;
            ExecutionResult {
                message: "Success".to_string(),
                code: 200,
                collaboration_id: collab_id,
                secret_id: Some(res_id)
            }
        },
        Err(err) => {
            let err_message = err.to_string();
            collab_ops::set_result_failed(collab_id, err_message.clone(), db_url)?;
            ExecutionResult {
                message: err_message,
                code: 500,
                collaboration_id: collab_id,
                secret_id: None
            }
        }
    };
    notify_parties(output_parties, res).await?;
    Ok(())
}