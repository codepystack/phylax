use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::{
    db,
    error::{AppError, AppResult},
    models::User,
    state::AppState,
};

/// Auth context injected into protected request extensions.
#[derive(Clone)]
pub struct AuthUser {
    pub user: User,
    pub session_id: String,
    /// Decrypted vault key for this session (kept in memory only).
    pub vault_key: [u8; 32],
}

/// Auth middleware – validates the Bearer session token and injects `AuthUser`
/// as an extension on the request.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&req)?;
    let session = db::get_session(&state.pool, &token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let user = db::get_user_by_id(&state.pool, &session.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Decrypt the session-stored vault key using the server secret.
    let server_secret = server_secret();
    let vault_key_bytes = crate::crypto::decrypt(&server_secret, &session.encrypted_session_key)
        .map_err(|_| AppError::Unauthorized)?;

    if vault_key_bytes.len() != 32 {
        return Err(AppError::Unauthorized);
    }

    let mut vault_key = [0u8; 32];
    vault_key.copy_from_slice(&vault_key_bytes);

    req.extensions_mut().insert(AuthUser {
        user,
        session_id: token.to_string(),
        vault_key,
    });

    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request) -> AppResult<String> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if let Some(token) = header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(AppError::Unauthorized)
    }
}

/// Returns a 32-byte server secret derived from the SERVER_SECRET env var.
pub fn server_secret() -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let raw = std::env::var("SERVER_SECRET")
        .unwrap_or_else(|_| "change-me-in-production-server-secret-key".to_string());
    let hash = Sha256::digest(raw.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash);
    out
}
