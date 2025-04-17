use poem_openapi::types::ToJSON;
use reqwest::Client;
use tracing::{event, Level};

use crate::{cs_execute::ExecutionResult, error::Result};

pub async fn notify_parties(output_parties: Vec<String>, result: ExecutionResult) -> Result<()> {
    for party in output_parties {
        let response = Client::new()
            .put(format!("{}/notify", party))
            .body(result.to_json_string())
            .header("accepts", "application/json")
            .header("Content-Type","application/json")
            .send().await?;
            if response.status().is_success() {
                event!(Level::INFO,"Output party {} was notified.", party)
            } else {
                let body = response.bytes().await?.to_vec();
                let s = String::from_utf8_lossy(&body);
                event!(Level::WARN,"Unable to notify output party {}", s)
            }
    }
    Ok(())
}
