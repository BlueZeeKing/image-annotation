use axum::{response::IntoResponse, routing::get, Router};
use image_annotation::{
    api::{get_random_image, get_router, ImageState},
    templates::{Help, NoImages, Upload},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let router = Router::new()
        .route("/", get(get_random_image))
        .route("/upload", get(upload))
        .route("/help", get(help))
        .route("/no_images", get(no_images))
        .nest("/api", get_router().await)
        .nest_service("/public", ServeDir::new("public"))
        .with_state(ImageState::new().await.unwrap());

    info!("Listening on {}", ADDR);

    let listener = TcpListener::bind(ADDR).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

pub async fn upload() -> impl IntoResponse {
    Upload {}
}

pub async fn no_images() -> impl IntoResponse {
    NoImages {}
}

pub async fn help() -> impl IntoResponse {
    Help {}
}
