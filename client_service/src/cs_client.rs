use mockall::predicate::*;
use mockall::*;
use poem_openapi::Object;
use serde::Serialize;

use crate::error::Result;
use crate::netaccess::NetAccess;


/// Mockable cs client interface
#[automock]
pub trait CsClient {
    async fn create_secrets<'a>(&self, secrets: Vec<&'a str>, uuid: Option<String>) -> Result<Vec<String>>;
    async fn delete_secrets(&self, secrets: Vec<String>) -> Result<String>;
    async fn get_secret(&self, secret_id: &str) -> Result<ClearTextSecret>;
    async fn list_secrets(&self) -> Result<Vec<String>>;
}

#[derive(Serialize, Object, Debug)]
pub struct ClearTextSecret {
    pub result: String,
    // pub creation_date: Option<String>,
    // pub game_id: Option<String>
}

// export JavaCsClient
pub use java_cs_client::JavaCsClient;
mod java_cs_client {
    use std::{env, ffi::OsStr, process::Command, sync::Mutex};
    use crate::{cs_config::{get_config, CarbynestackConfig}, error::Error};

    use super::*;
    use lazy_static::lazy_static;

    // used to prevent race condition on different cs-configs.
    lazy_static! {
        static ref CONFIG_LOCK: Mutex<i32> = Mutex::new(0i32);
    }

    fn jar_location() -> String {
        match env::var("CS_JAR_LOCATION") {
            Ok(addr) => addr,
            Err(_) => "/usr/local/cs.jar".to_string()
        }
    }

    pub struct JavaCsClient {
        config: CarbynestackConfig
    }

    struct SecretUtils {}
    impl SecretUtils {
        pub fn parse_secret(resp: String) -> ClearTextSecret {
            let strings = resp.split("\n").collect::<Vec<&str>>();
            let res = ClearTextSecret {
                result: String::from(strings[0].replace("[", "").replace("]","")),
                // creation_date: None,
                // game_id: None,
            };
            // for s in strings {
            //     let s = String::from(s.trim());
            //     if s.starts_with("creation-date") {
            //         res.creation_date = Some(s.replace("creation-date -> ", ""))
            //     } else if s.starts_with("gameID") {
            //         res.game_id = Some(s.replace("gameID -> ", ""))
            //     }
            // }
            res
        }
    }

    impl JavaCsClient {
        pub async fn new(collab_id: i32, net: &impl NetAccess) -> Result<JavaCsClient> {
            let config = get_config(collab_id, net).await?;
            Ok(JavaCsClient {
                config
            })
        }

    }

    struct CsCommand {
        command: Command
    }


    impl CsCommand {
        fn new() -> CsCommand {
            let mut command = Command::new("java");
            command.arg("-jar")
                .arg(jar_location());
            CsCommand {
                command
            }
        }
        fn amphora() -> CsCommand {
            let mut cscommand = CsCommand::new();
            cscommand.command.arg("amphora");
            cscommand
        }
        fn arg<S>(mut self, arg: S) -> CsCommand
        where
            S: AsRef<OsStr>, 
            {
            self.command.arg(arg);
            self
        }
        fn args<I,S>(mut self, args: I)  -> CsCommand
            where I: IntoIterator<Item = S>,
            S: AsRef<OsStr>

         {
            self.command.args(args);
            self
        }

        // returns result of stdout if successful
        fn output(mut self) -> Result<String> {
            match self.command.output() {
                Ok(output) => {
                    let stderr = String::from_utf8(output.stderr)
                        .map_err(|err| Error::from(err.to_string()))?;
                    if stderr != "" {
                        return Err(Error::from(stderr));
                    }
                    if output.status.success() {
                        return Ok(String::from_utf8(output.stdout)
                            .map_err(|err| err.to_string())?);
                    } else {
                        return Err(Error::from(format!("Unable to execute command {:?}", self.command)));
                    }
                },
                Err(err) => Err(Error::from(err)),
            }
        }
    }


    impl CsClient for JavaCsClient {
        /// Create a new secret
        async fn create_secrets<'a>(&self, secrets: Vec<&'a str>, uuid: Option<String>) -> Result<Vec<String>> {
            self.config.save_config_json()?;
            let output = if let Some(uuid) = uuid {
                CsCommand::amphora()
                    .arg("create-secret")
                    .arg("--secret-id")
                    .arg(uuid)
                    .args(secrets)
                    .output()?
            } else {
                CsCommand::amphora()
                    .arg("create-secret")
                    .args(secrets)
                    .output()?
            };
            let res: Vec<String> = vec![output.replace("\n", "")];
            Ok(res)
        }

        /// Delete secrets specified by secret_ids.
        async fn delete_secrets(&self, secret_ids: Vec<String>) -> Result<String> {
            self.config.save_config_json()?;
            let output = CsCommand::amphora()
                .arg("delete-secrets")
                .args(secret_ids)
                .output()?;
            Ok(output)
        }

        /// Get secret by id
        async fn get_secret(&self, secret_id: &str) -> Result<ClearTextSecret> {
            let output = CsCommand::amphora()
                .arg("get-secret")
                .arg(secret_id)
                .output()?;
            Ok(SecretUtils::parse_secret(output))
        }
        
        /// List all secret ids
        async fn list_secrets(&self) -> Result<Vec<String>> {
            let output = CsCommand::amphora()
                .arg("get-secrets")
                .arg("-l")
                .output()?;
            Ok(output.lines().map(|s| String::from(s)).collect::<Vec<String>>())
        }
    }
    #[cfg(test)]
    mod test{
        use super::*;

        #[test]
        fn test_parse_result() {
            let test_input = "[052deaa9-b4d7-440d-a44d-e0241cef21ff]\ngameID -> c3f5c561-2790-48fc-a490-63989272a2a\ncreation-date -> 2024/10/30";
            let parsed = SecretUtils::parse_secret(test_input.to_string());
            assert_eq!(parsed.result, "052deaa9-b4d7-440d-a44d-e0241cef21ff");
        }
    }
}
