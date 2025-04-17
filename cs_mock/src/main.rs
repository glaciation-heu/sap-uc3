use poem::{listener::TcpListener, Route, Server, EndpointExt, middleware::Cors};
use poem_openapi::OpenApiService;
use tracing::{event, Level};
use std::env;

mod api;

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt().with_max_level(Level::DEBUG)
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

    event!(Level::INFO, "Starting cs mock on {}:{}", addr, port);

    let api_service =
        OpenApiService::new((api::amphora::AmphoraApi, api::ephemeral::EphemeralApi), "Carbynestack Mock", "1.0")
            .description("A mock of the CarbyneStack services, used for testing")
            .server(oas_server);

    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest(format!("{}/", &prefix), api_service)
        .nest(format!("{}/docs", &prefix), ui)
        .with(Cors::new());

    let _ = Server::new(TcpListener::bind(format!("{}:{}", addr, port)))
        .run(app)
        .await;
}
