use poem_openapi::{OpenApi, OpenApiService};

pub mod middlewares;
mod tags;
mod token;
mod user;

pub fn create_api_service() -> OpenApiService<impl OpenApi, ()> {
    OpenApiService::new(token::ApiToken, "Love & Dream", env!("CARGO_PKG_VERSION"))
}
