
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::error::Result;
use super::{models::{CsConfig, CsProvider, NewCsConfig}, establish_connection};



/// Create new csconfig 
pub fn create(config: NewCsConfig, db_url: &str) -> Result<CsConfig> {
    use crate::schema::csconfig;

    let mut connection = establish_connection(db_url)?;
    let config = diesel::insert_into(csconfig::table)
        .values(&config)
        .get_result(&mut connection)?;
    Ok(config)
}

pub fn get(id_config: i32, db_url: &str) -> Result<CsConfig> {
    use crate::schema::csconfig::dsl::*;
    let mut connection = establish_connection(db_url)?;

    let config = csconfig.find(id_config)
        .first(&mut connection)?;
    Ok(config)
}

pub fn get_providers(id_config: i32, db_url: &str) -> Result<Vec<CsProvider>> {
    use crate::schema::csprovider;
    let mut connection = establish_connection(db_url)?;

    let providers = csprovider::dsl::csprovider.filter(csprovider::dsl::config_id.eq(id_config))
        .get_results::<CsProvider>(&mut connection)?;
    Ok(providers)
}

/// Create new participation between user and collaboration
pub fn create_providers(providers: Vec<CsProvider>, db_url: &str) -> Result<()> {
    use crate::schema::csprovider;

    let mut connection = establish_connection(db_url)?;
    for p in providers {
        diesel::insert_into(csprovider::table)
            .values(&p)
            .get_result::<CsProvider>(&mut connection)?;
    }
    Ok(())
}