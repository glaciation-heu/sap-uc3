
#[cfg(test)]
use std::env;

use diesel::{Connection, PgConnection, RunQueryDsl};
use poem::{middleware::AddDataEndpoint, test::{TestClient, TestForm, TestFormField, TestResponse}, EndpointExt, Route};
use poem_openapi::OpenApiService;
use coordination_service::db::{self, establish_connection};
use rand::Rng;
use tempfile::NamedTempFile;

use std::{io::Write, iter};

pub fn test_client(db_url: &str) -> TestClient<AddDataEndpoint<Route, std::string::String>> {

    let api_service = OpenApiService::new(
        (
            coordination_service::api::collaboration::CollabApi,
            coordination_service::api::sys_status::SysStatusApi,
        ),
        "",
        "1.0",
    );
    let database_ur = db_url.to_string();
    let app = Route::new()
        .nest("/", api_service)
        .data(database_ur);
    TestClient::new(app)
}

// Keep the databse info in mind to drop them later
pub struct DBTestContext {
    base_url: String,
    db_name: String,
    pub db_url: String
}

impl DBTestContext {
        pub fn new() -> Self {
            // First, connect to postgres db to be able to create our test
            // database.
            // DATABASE_HOST must be set.
            let db_host = env::var("DATABASE_HOST").expect("DATABASE_HOST environment variable not set.");
            // DATABASE_USER must be set.
            let db_usr = env::var("DATABASE_USER").expect("DATABASE_USER environment variable not set.");
            // DATABASE_PASSWD must be set.
            let db_pwd = env::var("DATABASE_PASSWD").expect("DATABASE_PASSWD environment variable not set.");
            let base_url = format!("postgres://{}:{}@{}", db_usr, db_pwd, db_host);
            let postgres_url= format!("{}/postgres", base_url);
            let mut conn = PgConnection::establish(&postgres_url).expect("Unable to connect to postgres test database.");

            let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
            let mut rng = rand::thread_rng();
            let one_char = || charset[rng.gen_range(0..charset.len())] as char;
            let db_name: String = iter::repeat_with(one_char).take(16).collect();
            let create_db_query = diesel::sql_query(format!("CREATE DATABASE {};", db_name).as_str());
            create_db_query.execute(&mut conn).expect(format!("Could not create database {}", db_name).as_str());

            // create random db name

            let db_url = format!("{}/{}", base_url, db_name);
            let mut connection = establish_connection(&db_url)
                .expect("Unable to connect to test database");
            db::run_pending_migrations(& mut connection).expect("Unable to run db migrations");

            Self {
                    base_url: base_url.to_string(),
                    db_name: db_name.to_string(),
                    db_url: db_url
            }
        }        
}

impl Drop for DBTestContext {
    fn drop(&mut self) {
        let postgres_url = format!("{}/postgres", self.base_url);
        let mut conn = PgConnection::establish(&postgres_url).expect("Cannot connect to postgres database.");
        let disconnect_users = format!(
            "SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '{}';",
            self.db_name
        );
        diesel::sql_query(disconnect_users.as_str()).execute(&mut conn).unwrap();
        let query = diesel::sql_query(format!("DROP DATABASE {};", self.db_name).as_str());
        query.execute(&mut conn)
            .expect(format!("Couldn't drop database {}", self.db_name).as_str());
    }
}


pub async fn create_correct_collaboration(client: &TestClient<AddDataEndpoint<Route, std::string::String>>) -> TestResponse {
    let mut tmp_program = NamedTempFile::new().unwrap();
    let mut tmp_config = NamedTempFile::new().unwrap();

    tmp_program.write_all(b"this is just some data").unwrap();

    // prime is missing in config
    tmp_config.write_all(
    br#"{
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
    "rinv":"133854242216446749056083838363708373830"}"#
    ).unwrap();

    let program_file = tokio::fs::File::open(tmp_program.path().to_owned()).await.unwrap();
    let config_file = tokio::fs::File::open(tmp_config.path().to_owned()).await.unwrap();

    let program_field = TestFormField::async_reader(tokio::io::BufReader::new(program_file))
        .filename("mpc_program")
        .name("mpc_program");
    let config_field = TestFormField::async_reader(tokio::io::BufReader::new(config_file))
        .filename("cs_config")
        .name("cs_config");
    client.post("/collaboration")
        .multipart(TestForm::new()
            .field(TestFormField::text("demo").name("name"))
            .field(TestFormField::text("data").name("csv_header_line"))
            .field(TestFormField::text("1").name("number_of_parties"))
            .field(program_field)
            .field(config_field)
        ).send().await
}