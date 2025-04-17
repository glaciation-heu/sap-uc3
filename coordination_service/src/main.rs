mod api;
mod db;
mod cs_definitions;
mod schema;
mod cs_execute;
mod notification_service;
mod error;

use poem::{listener::TcpListener, Route, Server, EndpointExt, middleware::Cors};
use poem_openapi::OpenApiService;
use tracing::{event, Level};
use std::env;
use error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()>{
    // Setup the database
   dotenv().ok();
    // DATABASE_HOST must be set.
    let db_host = env::var("DATABASE_HOST")?;
    // DATABASE_USER must be set.
    let db_usr = env::var("DATABASE_USER")?;
    // DATABASE_PASSWD must be set.
    let db_pwd = env::var("DATABASE_PASSWD")?;
    // DATABASE_DBNAME must be set.
    let db_name = env::var("DATABASE_DBNAME")?;
    let database_url = format!("postgres://{}:{}@{}/{}", db_usr, db_pwd, db_host, db_name);

    // Run diesel migrations
    let mut connection = db::establish_connection(&database_url)?;
    db::run_pending_migrations(&mut connection)?;

    let loglevel = match env::var("LOG_LEVEL") {
        Ok(level) => match level.as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::DEBUG
        },
        Err(_) => Level::DEBUG
    };

    tracing_subscriber::fmt()
        .with_max_level(loglevel)
        .compact()
        //.without_time()
        .with_target(false)
        .init();

    let port = match env::var("SERVICE_PORT") {
        Ok(port) => port,
        Err(_) => "80".to_string()
    };

    let addr = match env::var("SERVICE_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => "0.0.0.0".to_string()
    };

    let oas_server = match env::var("SWAGGER_SERVER_URI") {
        Ok(addr) => addr,
        Err(_) => format!("http://{}:{}", addr, port)
    };

    let prefix = match env::var("URL_PREFIX") {
        Ok(prefix) => prefix,
        Err(_) => "".to_string()
    };

    event!(Level::INFO, "Starting coordination service on {}:{}", addr, port);

    let api_service =
        OpenApiService::new((api::collaboration::CollabApi, api::sys_status::SysStatusApi), "Coordination Service", "1.0")
            .description("Coordination Service to coordinate MPC execution")
            .server(oas_server);

    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest(format!("{}/", &prefix), api_service)
        .nest(format!("{}/docs", &prefix), ui)
        .data(database_url)
        .with(Cors::new());

    let _ = Server::new(TcpListener::bind(format!("{}:{}", addr, port)))
        .run(app)
        .await;
    Ok(())
}
