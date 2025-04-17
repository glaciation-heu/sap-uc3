use serde::{Deserialize, Serialize};

use poem_openapi::Object;

#[derive(Object, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct CsProviderConfig {
    amphora_service_url: String,
    castor_service_url: String,
    ephemeral_service_url: String,
    id: i32,
    base_url: String
}

#[derive(Object, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct CsConfig {
    prim: String,
    r: String,
    rinv: String,
    no_ssl_validation: bool,
    trusted_certificates: Vec<String>,
    providers: Vec<CsProviderConfig>
}