use axum::{middleware, routing::{delete, get, post, put}, Router};

use crate::{auth::auth_middleware, state::AppState};

mod auth_routes;
mod vault_routes;

pub fn api_router(state: AppState) -> Router<AppState> {
    // Public routes
    let public_auth = Router::new()
        .route("/auth/register", post(auth_routes::register))
        .route("/auth/login", post(auth_routes::login))
        .route("/generate-password", post(vault_routes::generate_password_handler));

    // Protected routes (require valid session token)
    let protected = Router::new()
        .route("/auth/logout", post(auth_routes::logout))
        .route("/vault/entries", get(vault_routes::list_entries))
        .route("/vault/entries", post(vault_routes::create_entry))
        .route("/vault/entries/{id}", get(vault_routes::get_entry))
        .route("/vault/entries/{id}", put(vault_routes::update_entry))
        .route("/vault/entries/{id}", delete(vault_routes::delete_entry))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    Router::new()
        .merge(public_auth)
        .merge(protected)
}


