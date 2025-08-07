mod api;
mod error;

use poem::{listener::TcpListener, Route, Server, EndpointExt, middleware::Cors};
use poem_openapi::OpenApiService;
use tracing::{event, Level};
use std::env;



#[tokio::main]
async fn main() {

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
        .compact()
        //.without_time()
        .with_target(false)
        .with_max_level(loglevel)
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
    let coord_uri = match env::var("COORDINATOR_URI") {
        Ok(uri) => uri,
        Err(e) => panic!("Unable to get COORDINATOR_URI environment variable")
    };

    event!(Level::INFO, "Starting client service on {}:{}", addr, port);

    let party_name = match env::var("PARTY_NAME") {
        Ok(name) => name,
        Err(_) => "".to_string()
    };

    let api_service =
        OpenApiService::new(api::Api, format!("Client Service {}", party_name), "1.0")
            .description("Client Service to access the computation service.")
            .server(oas_server);

    let ui = api_service.swagger_ui();
    let spec_endpoint = api_service.spec_endpoint_yaml();
    let app = Route::new()
        .nest(format!("{}/", &prefix), api_service)
        .nest(format!("{}/docs", &prefix), ui)
        .nest(format!("{}/docs/spec", &prefix), spec_endpoint)
        .data(coord_uri)
        .with(Cors::new());

    let _ = Server::new(TcpListener::bind(format!("{}:{}", addr, port)))
        .run(app)
        .await;
}

//#[cfg(test)]
//mod tests {
//    #[test]
//}
