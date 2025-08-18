#[cfg(test)]
use std::env;

use poem::{test::{TestClient, TestForm, TestFormField, TestResponse}, EndpointExt, Route};
use poem_openapi::OpenApiService;
use reqwest::{multipart::{Form, Part}, Client};
use serde::Deserialize;
use tempfile::NamedTempFile;

use std::io::Write;

fn coord_uri() -> String {
    match env::var("COORDINATOR_URI") {
        Ok(uri) => uri,
        Err(_) => "http://localhost:8081".to_string()
    }
}

pub fn test_client() -> TestClient<Route> {
    let api_service = OpenApiService::new(client_service::api::Api, "", "1.0").data(coord_uri());
    let app = Route::new().nest("/", api_service);
    TestClient::new(app)
}

#[derive(Deserialize)]
pub struct Collaboration {
    pub id: i32,
    /// Name of the collaboration
    pub name: String,
    /// SPDZ Programm (base64 encoded)
    pub mpc_program: String,
    /// input parameter cs specification (base64 encoded)
    pub csv_specification: String,
    // Number of participating parties
    pub participation_number: i32,
    // id of cs configuration
    pub config_id: i32,
    // endpoints of output parties
    pub output_parties: Option<Vec<Option<String>>>
}

pub struct EnvConfig {
    test_collab_id: i32
}

pub async fn setup_env() -> Collaboration {
    // get existing collaborations for the test environment
    let client = Client::new();   
    //env::set_var("COORDINATOR_URI", "http://localhost:8082");
    let collabs = client.get(format!("{}/collaboration", coord_uri())).send().await.unwrap();
    let collabs = collabs.json::<Vec<Collaboration>>().await.unwrap();
    // remove previous collaborations
    for c in collabs {
        let _ = client.delete(format!("{}/collaboration/{}", coord_uri(), c.id)).send().await;
    }
     let config = Part::bytes(br#"
 {
   "prime" : "198766463529478683931867765928436695041",
   "r" : "141515903391459779531506841503331516415",
   "noSslValidation" : true,
   "providers" : [ {
     "amphoraServiceUrl" : "http://csmock/0/amphora",
     "castorServiceUrl" : "http://csmock/0/castor",
     "ephemeralServiceUrl" : "http://csmock/0",
     "id" : 1,
     "baseUrl" : "http://csmock/0"
   }, {
     "amphoraServiceUrl" : "http://csmock/1/amphora",
     "castorServiceUrl" : "http://csmock/1/castor",
     "ephemeralServiceUrl" : "http://csmock/1",
     "id" : 2,
     "baseUrl" : "http://csmock/1"
   } ],
   "rinv" : "133854242216446749056083838363708373830"
 }
 "#).file_name("cs_config");
//     let config = Part::bytes(br#"
// {
//   "prime" : "198766463529478683931867765928436695041",
//   "r" : "141515903391459779531506841503331516415",
//   "noSslValidation" : true,
//   "providers" : [ {
//     "amphoraServiceUrl" : "http://localhost:8085/0/amphora",
//     "castorServiceUrl" : "http://localhost:8085/0/castor",
//     "ephemeralServiceUrl" : "http://localhost:8085/0",
//     "id" : 1,
//     "baseUrl" : "http://localhost:8085/0"
//   }, {
//     "amphoraServiceUrl" : "http://localhost:8085/1/amphora",
//     "castorServiceUrl" : "http://localhost:8085/1/castor",
//     "ephemeralServiceUrl" : "http://localhost:8085/1",
//     "id" : 2,
//     "baseUrl" : "http://localhost:8085/1"
//   } ],
//   "rinv" : "133854242216446749056083838363708373830"
// }
// "#).file_name("cs_config");
    let program = Part::bytes(br#"
# Prologue to read in the inputs
listen_for_clients(PORTNUM)
port=regint(10000)
listen(port)
socket_id = regint()
acceptclientconnection(socket_id, port)
v = sint.read_from_socket(socket_id, 2)
val1 = v[0]
val2 = v[1]

# The logic
result = val1 < val2 

# Epilogue to return the outputs 
resp = Array(1, sint)
resp[0] = result
sint.write_to_socket(socket_id, resp)
"#)
    .file_name("mpc_program");
    // create test collaboration
    let form = Form::new()
        .text("name", "testing")
        .text("csv_header_line", "data")
        .text("number_of_parties", "1")
        .part("mpc_program", program)
        .part("cs_config", config);
    let collab = client.post(format!("{}/collaboration", coord_uri()))
    .multipart(form)
    .send().await.unwrap();
    collab.json::<Collaboration>().await.unwrap()
}

pub async fn register_input_party(collab_id: i32, party_id: i32) {
    let client = Client::new();   
    let _ = client
        .post(format!("{}/collaboration/{}/register-input-party/{}", coord_uri(), collab_id, party_id))
        .send().await.unwrap();
}

pub async fn upload_secret(client: &TestClient<Route>, collab_id: i32, party_id: i32, secret_id: Option<String>) -> TestResponse {
    // Create a temporary file with some content
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"data\n22").unwrap();
    let file_path = temp_file.path().to_owned();

    let file = tokio::fs::File::open(file_path.clone()).await.unwrap();

    let field = TestFormField::async_reader(tokio::io::BufReader::new(file))
        .filename("data_csv")
        .name("data_csv");

    let request_url = match secret_id {
        Some(id) => format!("/secrets/{}/{}?secret_id={}", collab_id, party_id, id),
        None => format!("/secrets/{}/{}", collab_id, party_id)
    };

    client.post(request_url)
        .multipart(TestForm::new().field(field))
        .send()
        .await
}