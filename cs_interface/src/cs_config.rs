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
        let home = env::var("HOME")?;
        let path = std::path::Path::new(&home).join(".cs");
        std::fs::create_dir_all(path.clone())?;
        event!(Level::DEBUG, "Saving cs-config to file {}", &path.display());

        // create config file at home location
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.join("config"))?;
        let mut writer = BufWriter::new(f);
        serde_json::to_writer(&mut writer, self)
            .map_err(|err| Error::from(err))?;
        writer.flush()?;
        Ok(())
    } 

    pub fn from_json(json: &str) -> Result<CarbynestackConfig> {
        let json = serde_json::from_str(json)?;
        Ok(json)
    }
    pub async fn get_from_coordinator(coord_url: &str, collaboration_id: i32, net: &impl NetAccess) -> Result<CarbynestackConfig> {
        let url = format!("{}/collaboration/{}/compute_config", coord_url, collaboration_id);
        let bytes = net.get(&url).await?;
        let data = String::from_utf8(bytes)?;
        CarbynestackConfig::from_json(&data)
    }
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
       let config = CarbynestackConfig::get_from_coordinator("http://coordinator", 1, &mock_server).await?;
       config_assertions(config);
       mock_server.checkpoint();
       Ok(())
   }

   #[tokio::test]
   async fn test_get_config_error() {
       let mut mock_server = MockNetAccess::new();
       mock_server.expect_get()
           .times(1)
           .returning(|_| Err(Error::HttpError { code: 404, message: "Not found".to_string() }))
           .withf(|url| url == "http://coordinator/collaboration/1/compute_config");
       let resp = CarbynestackConfig::get_from_coordinator("http://coordinator", 1, &mock_server).await;
       match resp {
           Ok(_) => assert!(false, "Expected error response"),
           Err(_) => assert!(true),
       }
    }

    #[test]
    fn test_save_config() -> Result<()> {
        let test_dir = TempDir::new("testhome").map_err(|err| Error::Io(err))?;
        let path = test_dir.path();
        unsafe {
            env::set_var("HOME", path.display().to_string());
        }
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
