use crate::error::Result;
use mockall::predicate::*;
use mockall::*;
use poem_openapi::Object;
use serde::Serialize;

/// Mockable cs client interface
#[automock]
pub trait CsClient {
    fn create_secrets<'a>(
        &self,
        secrets: Vec<&'a str>,
        uuid: Option<String>,
    ) -> Result<Vec<String>>;
    fn delete_secrets(&self, secrets: Vec<String>) -> Result<String>;
    fn get_secret(&self, secret_id: &str) -> Result<ClearTextSecret>;
    fn list_secrets(&self) -> Result<Vec<String>>;
    fn get_comp_party_urls(&self) -> Vec<String>;
    fn execute_program(&self, spdz_program: String, secret_ids: Vec<String>) -> Result<String>;
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
    use crate::{CS_JAR, cs_config::CarbynestackConfig, error::Error};
    use std::{env, ffi::OsStr, fs, io::Write, process::{Child, Command, Stdio}, sync::Mutex};

    use super::*;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use jni::InitArgsBuilder;
    use lazy_static::lazy_static;
    use tempfile::NamedTempFile;
    use tracing::{event, Level};

    // used to prevent race condition on different cs-configs.
    lazy_static! {
        static ref CONFIG_LOCK: Mutex<i32> = Mutex::new(0i32);
    }

    fn jar_location() -> String {
        match env::var("CS_JAR_LOCATION") {
            Ok(addr) => addr,
            Err(_) => "/usr/local/cs.jar".to_string(),
        }
    }

    pub struct JavaCsClient {
        config: CarbynestackConfig,
    }

    struct SecretUtils {}
    impl SecretUtils {
        pub fn parse_secret(resp: String) -> ClearTextSecret {
            let strings = resp.split("\n").collect::<Vec<&str>>();
            let res = ClearTextSecret {
                result: String::from(strings[0].replace("[", "").replace("]", "")),
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
        pub fn new(config: CarbynestackConfig) -> Result<JavaCsClient> {
            Ok(JavaCsClient { config })
        }
    }

    struct CsCommand {
        command: Command,
    }

    impl CsCommand {
        fn new() -> CsCommand {
            //let mut tmp_jar = NamedTempFile::new()?;
            //fs::write(&mut tmp_jar, CS_JAR)?;
            //let jar_path = tmp_jar.path().to_str().ok_or(Error::CommandError(
            //    "Unable to open cs.jar path".to_string(),
            //))?;
            //let jvm_args = InitArgsBuilder::new()
            //    .version(jni::JNIVersion::V8)
            //    .option("-Djava.class.path=".to_owned() + jar_path);
            let mut command = Command::new("java");
            command.arg("-jar").arg(jar_location());
            CsCommand { command }
        }
        fn amphora() -> CsCommand {
            let mut cscommand = CsCommand::new();
            cscommand.command.arg("amphora");
            cscommand
        }
        fn ephemeral() -> CsCommand {
            let mut cscommand = CsCommand::new();
            cscommand.command.arg("ephemeral");
            cscommand
        }
        fn arg<S>(mut self, arg: S) -> CsCommand
        where
            S: AsRef<OsStr>,
        {
            self.command.arg(arg);
            self
        }
        fn args<I, S>(mut self, args: I) -> CsCommand
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>,
        {
            self.command.args(args);
            self
        }

        fn std_piped(mut self) -> Self {
            self.command.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            self
        }

        // returns result of stdout if successful
        fn output(mut self) -> Result<String> {
            match self.command.output() {
                Ok(output) => {
                    let stderr = String::from_utf8(output.stderr)?;
                    if stderr != "" {
                        return Err(Error::CommandError(stderr));
                    }
                    if output.status.success() {
                        return Ok(String::from_utf8(output.stdout)?);
                    } else {
                        return Err(Error::CommandError(format!(
                            "Unable to execute command {:?} status {:?}",
                            self.command,
                            output.status
                        )));
                    }
                }
                Err(err) => Err(Error::from(err)),
            }
        }
        fn spawn(mut self) -> Result<Child> {
            Ok(self.command.spawn()?)
        }
    }

    impl CsClient for JavaCsClient {
        fn get_comp_party_urls(&self) -> Vec<String> {
            self.config.providers.iter().map(|p| p.base_url.clone()).collect()
        }
        /// Create a new secret
        fn create_secrets<'a>(
            &self,
            secrets: Vec<&'a str>,
            uuid: Option<String>,
        ) -> Result<Vec<String>> {
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
        fn delete_secrets(&self, secret_ids: Vec<String>) -> Result<String> {
            self.config.save_config_json()?;
            let output = CsCommand::amphora()
                .arg("delete-secrets")
                .args(secret_ids)
                .output()?;
            Ok(output)
        }

        /// Get secret by id
        fn get_secret(&self, secret_id: &str) -> Result<ClearTextSecret> {
            let output = CsCommand::amphora()
                .arg("get-secret")
                .arg(secret_id)
                .output()?;
            Ok(SecretUtils::parse_secret(output))
        }

        /// List all secret ids
        fn list_secrets(&self) -> Result<Vec<String>> {
            let output = CsCommand::amphora().arg("get-secrets").arg("-l").output()?;
            Ok(output
                .lines()
                .map(|s| String::from(s))
                .collect::<Vec<String>>())
        }

        fn execute_program(&self, spdz_program:String, secret_ids:Vec<String>) -> Result<String> {
            let program = BASE64_STANDARD.decode(spdz_program)?;
            self.config.save_config_json()?;
            let mut java_execute = CsCommand::ephemeral()
                .arg("execute")
                .args(secret_ids.into_iter().map(|id| ("-i".to_string(), id)).flat_map(|tup| [tup.0, tup.1].clone()).collect::<Vec<String>>())
                .arg("ephemeral-generic.default")
                .std_piped()
                .spawn()?;
            let mut stdin = java_execute.stdin.take().expect("Failed to open stdin");
            std::thread::spawn(move || {
                stdin.write_all(&program).expect("Failed to write to stdin");
            });
            let output = java_execute.wait_with_output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout)
                    .replace("Provide program to execute. Press Ctrl+D to submit.\n", "")
                    .replace("\n", "")
                    .replace("[", "")
                    .replace("]", "");
                event!(Level::DEBUG, "Try parsing {}", &stdout);
                event!(Level::INFO, "MPC Execution finished successfully.");
                Ok(stdout)
            } else {
                let message = format!("Error:\nstderr:\n{}\nstdout\n{}", String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout));
                event!(Level::ERROR, "MPC program execution failed: {}", &message);
                Err(crate::Error::CommandError(message))
            }
            

        }
    
    }
    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_parse_result() {
            let test_input = "[052deaa9-b4d7-440d-a44d-e0241cef21ff]\ngameID -> c3f5c561-2790-48fc-a490-63989272a2a\ncreation-date -> 2024/10/30";
            let parsed = SecretUtils::parse_secret(test_input.to_string());
            assert_eq!(parsed.result, "052deaa9-b4d7-440d-a44d-e0241cef21ff");
        }
    }
}
