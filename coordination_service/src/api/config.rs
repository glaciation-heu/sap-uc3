use std::io::{BufWriter, Write};

use cs_interface::CarbynestackProvider;
use poem_openapi::Object;
use serde::{Serialize, Deserialize};

use crate::db::{self, models::{CsProvider, NewCsConfig, CsConfig}};
use crate::error::Result;
type BigNumber = String;
pub use cs_interface::CarbynestackConfig;

/// Save the config to the corresponding collaboration
pub fn add_config(config: CarbynestackConfig, db_url: &str) -> Result<CsConfig> {
    let db_config = db::csconfig_ops::create(NewCsConfig{
        r: config.r.to_string(),
        rinv: config.rinv.to_string(),
        prime: config.prime.to_string(),
        no_ssl_validation: config.no_ssl_validation
    }, db_url)?;
    db::csconfig_ops::create_providers(config.providers.iter().map(|p| CsProvider {
        id: p.id,
        config_id: db_config.id,
        amphora_service_url: p.amphora_service_url.clone(),
        castor_service_url: p.castor_service_url.clone(),
        ephemeral_service_url: p.ephemeral_service_url.clone(),
        base_url: p.base_url.clone()
    }).collect(), db_url)?;

    Ok(db_config)
}

/// Get config of a specific collaboration
pub fn get_config(collab_id: i32, db_url: &str) -> Result<CarbynestackConfig> {
    let collab = db::collab_ops::get(collab_id, db_url)?;
    let db_config = db::csconfig_ops::get(collab.config_id, db_url)?;
    let db_providers = db::csconfig_ops::get_providers(db_config.id, db_url)?;
    Ok(CarbynestackConfig {
        prime: db_config.prime,
        r: db_config.r,
        rinv: db_config.rinv,
        no_ssl_validation: db_config.no_ssl_validation,
        providers: db_providers.iter().map(|p| CarbynestackProvider {
            id: p.id,
            amphora_service_url: p.amphora_service_url.clone(),
            castor_service_url: p.castor_service_url.clone(),
            ephemeral_service_url: p.ephemeral_service_url.clone(),
            base_url: p.base_url.clone()
        }).collect()
    })
}
