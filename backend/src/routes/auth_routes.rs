use axum::{extract::State, Extension, Json};
use serde_json::{json, Value};

use crate::{
    auth::{server_secret, AuthUser},
    crypto::{
        derive_wrapping_key, encrypt, hash_master_password,
        verify_master_password, VaultKey,
    },
    db,
    error::{AppError, AppResult},
    models::{AuthResponse, LoginRequest, RegisterRequest},
    state::AppState,
};

/// POST /api/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Rate limit
    if state.auth_limiter.check().is_err() {
        return Err(AppError::RateLimit);
    }

    // Validate input
    let username = payload.username.trim().to_string();
    if username.is_empty() || username.len() > 64 {
        return Err(AppError::BadRequest("Invalid username".into()));
    }
    if payload.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    // Check for existing user
    if db::get_user_by_username(&state.pool, &username).await?.is_some() {
        return Err(AppError::BadRequest("Username already taken".into()));
    }

    // Hash master password with Argon2id
    let password_hash = hash_master_password(&payload.password)?;

    // Generate a new random vault key
    let vault_key = VaultKey::generate();

    // We need a user ID before we can derive the wrapping key.
    // Pre-generate it.
    let user_id = uuid::Uuid::new_v4().to_string();

    // Derive a wrapping key from the master password + user_id
    let wrapping_key = derive_wrapping_key(&payload.password, &user_id)?;

    // Encrypt the vault key with the wrapping key
    let encrypted_vault_key = encrypt(&wrapping_key, &vault_key.0)?;

    // Persist user (use raw query to pre-supply the UUID)
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, encrypted_vault_key, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&username)
    .bind(&password_hash)
    .bind(&encrypted_vault_key)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // Create session
    let secret = server_secret();
    let enc_session_key = encrypt(&secret, &vault_key.0)?;
    let session = db::create_session(&state.pool, &user_id, &enc_session_key).await?;

    Ok(Json(AuthResponse {
        session_token: session.id,
        user_id,
        username,
    }))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Rate limit
    if state.auth_limiter.check().is_err() {
        return Err(AppError::RateLimit);
    }

    let username = payload.username.trim().to_string();

    let user = db::get_user_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_master_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    // Derive the wrapping key from the provided password + user_id
    let wrapping_key = derive_wrapping_key(&payload.password, &user.id)?;

    // Decrypt the vault key
    let vault_key_bytes = crate::crypto::decrypt(&wrapping_key, &user.encrypted_vault_key)
        .map_err(|_| AppError::Unauthorized)?;

    if vault_key_bytes.len() != 32 {
        return Err(AppError::Unauthorized);
    }

    // Encrypt vault key with server secret for session storage
    let secret = server_secret();
    let mut vault_key_arr = [0u8; 32];
    vault_key_arr.copy_from_slice(&vault_key_bytes);
    let enc_session_key = encrypt(&secret, &vault_key_arr)?;

    let session = db::create_session(&state.pool, &user.id, &enc_session_key).await?;

    Ok(Json(AuthResponse {
        session_token: session.id,
        user_id: user.id,
        username: user.username,
    }))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> AppResult<Json<Value>> {
    db::delete_session(&state.pool, &auth.session_id).await?;
    Ok(Json(json!({ "message": "Logged out" })))
}
