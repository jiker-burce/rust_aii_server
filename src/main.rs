use poem::{listener::TcpListener, EndpointExt, Route, Server};

mod api;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service = api::create_api_service().server("http://0.0.0.0:3000/api");
    // 开启Swagger UI
    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/doc", ui)
        .with(api::middlewares::JwtMiddleware);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
