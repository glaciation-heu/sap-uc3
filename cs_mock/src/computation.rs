use std::{process::Command, thread};
use core::time;
use num_bigint::BigInt;
use poem::Result;
use tracing::{event, Level};

const PROGRAM_SOURCE_DIR:&'static str = "/root/MP-SPDZ/Programs/Source";

// Run the computation
pub async fn run_computation(program: String, data: Vec<BigInt>) -> Result<Vec<i32>> {
    // First write the program
    let path = std::path::Path::new(PROGRAM_SOURCE_DIR).join("program.mpc");
    event!(Level::INFO, "Start executing computation");
    std::fs::write(path, program).expect("Error writing program");

    // Create certificates
    match Command::new("Scripts/setup-ssl.sh").arg("2").output() {
        Ok(output) => {
            let stderr = String::from_utf8(output.stderr).unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            event!(Level::WARN, "stderr: {}", stderr);
            event!(Level::WARN, "stdout: {}", stdout);
        },
        Err(e) => event!(Level::ERROR, "{}", e),
    }
    match Command::new("Scripts/setup-clients.sh").arg("1").output() {
        Ok(output) => {
            let stderr = String::from_utf8(output.stderr).unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            event!(Level::WARN, "stderr: {}", stderr);
            event!(Level::WARN, "stdout: {}", stdout);
        },
        Err(e) => event!(Level::ERROR, "{}", e),
    }

    // Now start the clients, they wait for a ping on port 734
    Command::new("nc").arg("-z").arg("party0").arg("734").output().expect("Error starting nc to trigger computation on party1");
    Command::new("nc").arg("-z").arg("party1").arg("734").output().expect("Error starting nc to trigger computation on party2");

    // Run the computation
    let data = data.into_iter().map(|d| d.to_string()).collect::<Vec<String>>();
    event!(Level::INFO, "Running with data: {:?}", data);
    let output = Command::new("python").arg("ExternalIO/client-interface.py").args(data.into_iter().map(|d| d.to_string())).output().expect("Expecting outputs to return stuff");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        if !stderr.is_empty() {
            event!(Level::ERROR, "stderr {}", stderr);
            return Ok(vec![])
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        match serde_json::from_str::<Vec<i32>>(&stdout) {
            Ok(data) => {
                event!(Level::INFO, "Computation successfully finished {:?}", data);
                return Ok(data)
            },
            Err(e) => {
                event!(Level::ERROR, "json parse error {}", e);
            },
        }
    } else {
        event!(Level::ERROR, "Computation did not finish successfully status: {:?}\n{}", output.status.code(), stderr);
    }
    Ok(vec![])
}