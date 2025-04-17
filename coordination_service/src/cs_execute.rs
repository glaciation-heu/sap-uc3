use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use base64::prelude::*;
use poem_openapi::Object;
use tracing::{event, Level};

use crate::{api::config, error::Result};

fn jar_location() -> String {
    match env::var("CS_JAR_LOCATION") {
        Ok(addr) => addr,
        Err(_) => "/usr/local/cs.jar".to_string()
    }
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub message: String,
    pub code: i32,
    pub collaboration_id: i32,
    pub secret_id: Option<String>
}

pub fn execute_program(spdz_program: String, collab_id: i32, secret_ids: Vec<String>, config_id: i32, db_url: &str) -> Result<(ExecutionResult, Option<Vec<String>>)> {

    // base64 decode program
    let program = BASE64_STANDARD.decode(spdz_program)?;

    // Save cs config to file
    let config = config::get_config(config_id, db_url)?;
    config.save_config_json()?;

    event!(Level::DEBUG,"Start execution of MPC Program with secret ids: {:?}", secret_ids);
    let mut jar_execute = Command::new("java")
        .arg("-jar")
        .arg(jar_location())
        .arg("ephemeral")
        .arg("execute")
        .args(secret_ids.into_iter().map(|id| ("-i".to_string(), id)).flat_map(|tup| [tup.0, tup.1].clone()).collect::<Vec<String>>())
        .arg("ephemeral-generic.default")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = jar_execute.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&program).expect("Failed to write to stdin");
    });
    let output = jar_execute.wait_with_output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout)
            .replace("Provide program to execute. Press Ctrl+D to submit.\n", "")
            .replace("\n", "")
            .replace("[", "")
            .replace("]", "");
        event!(Level::DEBUG, "Try parsing {}", &stdout);
        let execution_res = ExecutionResult{
            message: format!("{}", &stdout),
            code: 200,
            collaboration_id: collab_id,
            secret_id: Some(stdout.clone())
        };
        event!(Level::INFO, "MPC Execution finished successfully.");
        Ok((execution_res, Some(vec![stdout])))
    } else {
        let execution_res = ExecutionResult{
            message: format!("Error:\nstderr:\n{}\nstdout\n{}", String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout)),
            code: 500,
            collaboration_id: collab_id,
            secret_id: None
        };
        event!(Level::ERROR, "MPC program execution failed: {}", &execution_res.message);
        Ok((execution_res, None))
    }
}
