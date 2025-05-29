use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};

mod app_state;
mod handlers;
mod models;
mod services;
mod utils;

use app_state::AppState;
use handlers::{kline_handler, websocket_handler};
use services::{data_generator::DataGenerator, kline_service::KlineService, stream_service::StreamService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Create broadcast channel for transactions
    let (tx, _rx) = broadcast::channel(1000);

    // Initialize services
    let kline_service = Arc::new(KlineService::new());
    let stream_service = Arc::new(StreamService::new());
    let data_generator = Arc::new(DataGenerator::new());

    // Start data generator in background
    let generator_tx = tx.clone();
    let generator_kline_service = kline_service.clone();
    tokio::spawn(async move {
        data_generator.start_generation(generator_tx, generator_kline_service).await;
    });

    // Create app state
    let app_state = Arc::new(AppState {
        kline_service,
        stream_service,
        tx,
    });

    // Build router
    let app = Router::new()
        .route("/api/klines/:symbol", get(kline_handler::get_klines))
        .route("/ws/:symbol", get(websocket_handler::websocket_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    info!("Starting server on 0.0.0.0:3000");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
