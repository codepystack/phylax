mod auth;
mod crypto;
mod db;
mod error;
mod models;
mod routes;
mod state;

use axum::Router;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "phylax_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize app state (DB pool, rate limiters, etc.)
    let state = AppState::new().await?;

    // Run database migrations
    state.run_migrations().await?;

    // Build router
    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let api = routes::api_router(state.clone());

    Router::new()
        .nest("/api", api)
        // Serve the SvelteKit static build
        .fallback_service(tower_http::services::ServeDir::new("../frontend/build"))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap(),
                    "http://localhost:4173".parse::<axum::http::HeaderValue>().unwrap(),
                    "http://localhost:8080".parse::<axum::http::HeaderValue>().unwrap(),
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ])
                .allow_credentials(false),
        )
        .with_state(state)
}
