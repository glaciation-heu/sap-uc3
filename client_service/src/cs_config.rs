use poem_openapi::Object;
use serde::{Serialize, Deserialize};
use tracing::{event, Level};
use crate::error::{Error, Result};
use std::env;
use std::io::{BufWriter, Write};


use crate::netaccess::NetAccess;

type BigNumber = String;

#[derive(Object, Deserialize, Serialize, Debug)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct CarbynestackConfig {
    /// The Prime as used by the MPC backend
    pub prime: BigNumber,
    /// The auxiliary modulus R as used by the MPC backend
    pub r: BigNumber,
    /// The multiplicative inverse for the auxiliary modulus R as used by the MPC backend
    pub rinv: BigNumber,
    pub no_ssl_validation: bool,
    pub providers: Vec<CarbynestackProvider>
}

#[derive(Object, Deserialize, Serialize, Debug)]
#[oai(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct CarbynestackProvider {
    pub id: i32,
    pub amphora_service_url: String,
    pub castor_service_url: String,
    pub ephemeral_service_url: String,
    pub base_url: String
}

impl CarbynestackConfig {

    /// Save CarbyneStack config as json at $HOME/.cs/config
    pub fn save_config_json(&self) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|err| Error::from(format!("Error reading HOME env variable. Cause: {}", err)))?;
        let path = std::path::Path::new(&home).join(".cs");
        std::fs::create_dir_all(path.clone())?;
        event!(Level::DEBUG, "Saving cs-config to file {}", &path.display());

        // create config file at home location
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.join("config"))
            .map_err(|err| format!("Unable to create or open cs config file: cause {err}"))?;
        let mut writer = BufWriter::new(f);
        serde_json::to_writer(&mut writer, self)
            .map_err(|err| Error::from(err))?;
        writer.flush()
            .map_err(|err| format!("Unable to write cs config to $HOME/.cs/config. cause: {err}"))?;
        Ok(())
    } 

    pub fn from_json(json: &str) -> Result<CarbynestackConfig> {
        let json = serde_json::from_str(json)
            .map_err( |err| format!("Error parsing CarbyneStack config from json. Cause {err}"))?;
        Ok(json)
    }
}

/// Get config of collaboration from the coordinator
pub async fn get_config(collab_id: i32, netget: &impl NetAccess) -> Result<CarbynestackConfig> {
    let url = env::var("COORDINATOR_URI")
        .map_err(|_| "COORDINATOR_URI environment variable not set")?;
    let url = format!("{}/collaboration/{}/compute_config", url, collab_id);
    event!(Level::DEBUG, "Get config from coordinator {}", &url);
    let bytes = netget.get(&url.as_str()).await
        .map_err(|_| Error::CollaborationNotFound { collab_id: collab_id })?;
    let data = String::from_utf8_lossy(&bytes).into_owned();
    event!(Level::DEBUG, "Try parsing config\n {}", data);
    CarbynestackConfig::from_json(&data)
}

#[cfg(test)]
mod test {
    use std::fs;

    use tempdir::TempDir;

    use crate::netaccess::MockNetAccess;

    use super::*;

    fn get_json_valid() -> String {
        r#"{
        "noSslValidation":true,
        "prime":"198766463529478683931867765928436695041",
        "providers":[
            {"amphoraServiceUrl":"http://csmock/0/amphora",
            "baseUrl":"http://csmock/0/",
            "castorServiceUrl":"http://csmock/0/castor",
            "ephemeralServiceUrl":"http://csmock/0/",
            "id":1},
            {"amphoraServiceUrl":"http://csmock/1/amphora",
            "baseUrl":"http://csmock/1/",
            "castorServiceUrl":"http://csmock/1/castor",
            "ephemeralServiceUrl":"http://csmock/1/",
            "id":2}],
        "r":"141515903391459779531506841503331516415",
        "rinv":"133854242216446749056083838363708373830"}"#.to_string()
    }

    fn get_json_invalid() -> String {
        // noSslValidation is string, not boolean
        r#"{
        "noSslValidation":"true",
        "prime":"198766463529478683931867765928436695041",
        "providers":[
            {"amphoraServiceUrl":"http://csmock/0/amphora",
            "baseUrl":"http://csmock/0/",
            "castorServiceUrl":"http://csmock/0/castor",
            "ephemeralServiceUrl":"http://csmock/0/",
            "id":1},
            {"amphoraServiceUrl":"http://csmock/1/amphora",
            "baseUrl":"http://csmock/1/",
            "castorServiceUrl":"http://csmock/1/castor",
            "ephemeralServiceUrl":"http://csmock/1/",
            "id":2}],
        "r":"141515903391459779531506841503331516415",
        "rinv":"133854242216446749056083838363708373830"}"#.to_string()
    }

    fn config_assertions(config: CarbynestackConfig) {
        assert_eq!(config.no_ssl_validation, true);
        assert_eq!(config.prime, "198766463529478683931867765928436695041".to_string());
        assert_eq!(config.r, "141515903391459779531506841503331516415"); 
        assert_eq!(config.rinv, "133854242216446749056083838363708373830"); 
        assert_eq!(config.providers.len(), 2);
    }

    /// Tests creating CarbyneStackConfig from valid json
    #[test]
    fn test_from_json_valid() {
        
        match CarbynestackConfig::from_json(&get_json_valid()) {
            Ok(cs_config) => config_assertions(cs_config),
            Err(err) => panic!("{err}")
        }
    }

    /// Tests creating CabyneStackConfig from invalid json
    #[test]
    fn test_from_json_invalid() {
        match CarbynestackConfig::from_json(&get_json_invalid()) {
            Ok(_) => panic!("Expected error while parsing invalid json"),
            Err(_) => assert!(true) // Error is expected!
        }
    }

    #[tokio::test]
    async fn test_get_config() -> Result<()> {
        let mut mock_server = MockNetAccess::new();
        mock_server.expect_get()
            .times(1)
            .returning(|_| Ok(get_json_valid().as_bytes().to_vec()))
            .withf(|url| url == "http://coordinator/collaboration/1/compute_config");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let config = get_config(1, &mock_server).await?;
        config_assertions(config);
        mock_server.checkpoint();
        Ok(())
    }

    #[tokio::test]
    async fn test_get_config_error() {
        let mut mock_server = MockNetAccess::new();
        mock_server.expect_get()
            .times(1)
            .returning(|_| Err(Error::from("")))
            .withf(|url| url == "http://coordinator/collaboration/1/compute_config");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let resp = get_config(1, &mock_server).await;
        match resp {
            Ok(_) => assert!(false, "Expected error response"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_save_config() -> Result<()> {
        let test_dir = TempDir::new("testhome").map_err(|err| Error::Io(err))?;
        let path = test_dir.path();
        env::set_var("HOME", path.display().to_string());
        println!("{}", env::var("HOME").unwrap());
        let config = CarbynestackConfig::from_json(&get_json_valid()).unwrap();
        // save config to temporary direction
        config.save_config_json()?;

        // read config to test if the correct config was saved
        let config_path = path.join(".cs").join("config");
        let content = fs::read_to_string(&config_path).map_err(|err| Error::Io(err))?;
        let new_config = CarbynestackConfig::from_json(&content)?;
        config_assertions(new_config);
        Ok(())
    }
}
