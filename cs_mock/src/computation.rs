use std::process::Command;
use poem::Result;
use tracing::{event, Level};

const PROGRAM_SOURCE_DIR:&'static str = "/root/MP-SPDZ/Programs/Source";

// Run the computation
pub fn run_computation(program: String, nr_of_outputs: i32) -> Result<Vec<String>> {
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

    // Now start the clients
    Command::new("nc").arg("-z").arg("party0").arg("734").output().expect("Error starting nc to trigger computation on party1");
    Command::new("nc").arg("-z").arg("party1").arg("734").output().expect("Error starting nc to trigger computation on party2");

    // Run the computation
    let output = Command::new("python").arg("ExternalIO/client-interface.py").arg(nr_of_outputs.to_string()).output().expect("Expecting outputs to return stuff");
    event!(Level::WARN ,"{}", String::from_utf8_lossy(&output.stdout));
    event!(Level::WARN ,"{}", String::from_utf8_lossy(&output.stderr));
    Ok(vec![])
}