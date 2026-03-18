use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::AuthUser,
    crypto::{decrypt_str, encrypt_str, generate_password},
    db,
    error::{AppError, AppResult},
    models::{
        CreateEntryRequest, GeneratePasswordRequest, GeneratePasswordResponse, UpdateEntryRequest,
        VaultEntry,
    },
    state::AppState,
};

/// GET /api/vault/entries
pub async fn list_entries(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> AppResult<Json<Vec<VaultEntry>>> {
    let rows = db::list_vault_entries(&state.pool, &auth.user.id).await?;
    let vault_key = &auth.vault_key;

    let entries: AppResult<Vec<VaultEntry>> = rows
        .into_iter()
        .map(|row| {
            Ok(VaultEntry {
                id: row.id,
                title: decrypt_str(vault_key, &row.title_enc)?,
                username: row
                    .username_enc
                    .as_deref()
                    .map(|enc| decrypt_str(vault_key, enc))
                    .transpose()?,
                password: decrypt_str(vault_key, &row.password_enc)?,
                url: row
                    .url_enc
                    .as_deref()
                    .map(|enc| decrypt_str(vault_key, enc))
                    .transpose()?,
                notes: row
                    .notes_enc
                    .as_deref()
                    .map(|enc| decrypt_str(vault_key, enc))
                    .transpose()?,
                category: row.category,
                is_favorite: row.is_favorite,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect();

    Ok(Json(entries?))
}

/// GET /api/vault/entries/:id
pub async fn get_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<VaultEntry>> {
    let row = db::get_vault_entry(&state.pool, &id, &auth.user.id)
        .await?
        .ok_or(AppError::NotFound)?;

    let vault_key = &auth.vault_key;
    let entry = VaultEntry {
        id: row.id,
        title: decrypt_str(vault_key, &row.title_enc)?,
        username: row
            .username_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        password: decrypt_str(vault_key, &row.password_enc)?,
        url: row
            .url_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        notes: row
            .notes_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        category: row.category,
        is_favorite: row.is_favorite,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(Json(entry))
}

/// POST /api/vault/entries
pub async fn create_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreateEntryRequest>,
) -> AppResult<Json<VaultEntry>> {
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title is required".into()));
    }
    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".into()));
    }

    let vault_key = &auth.vault_key;
    let title_enc = encrypt_str(vault_key, payload.title.trim())?;
    let username_enc = payload
        .username
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| encrypt_str(vault_key, s))
        .transpose()?;
    let password_enc = encrypt_str(vault_key, &payload.password)?;
    let url_enc = payload
        .url
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| encrypt_str(vault_key, s))
        .transpose()?;
    let notes_enc = payload
        .notes
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| encrypt_str(vault_key, s))
        .transpose()?;
    let category = payload
        .category
        .as_deref()
        .unwrap_or("login")
        .to_string();
    let is_favorite = payload.is_favorite.unwrap_or(false);

    let row = db::create_vault_entry(
        &state.pool,
        &auth.user.id,
        &title_enc,
        username_enc.as_deref(),
        &password_enc,
        url_enc.as_deref(),
        notes_enc.as_deref(),
        &category,
        is_favorite,
    )
    .await?;

    let entry = VaultEntry {
        id: row.id,
        title: payload.title.trim().to_string(),
        username: payload.username.filter(|s| !s.is_empty()),
        password: payload.password,
        url: payload.url.filter(|s| !s.is_empty()),
        notes: payload.notes.filter(|s| !s.is_empty()),
        category,
        is_favorite,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(Json(entry))
}

/// PUT /api/vault/entries/:id
pub async fn update_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEntryRequest>,
) -> AppResult<Json<VaultEntry>> {
    // Verify entry exists and belongs to user
    let _existing = db::get_vault_entry(&state.pool, &id, &auth.user.id)
        .await?
        .ok_or(AppError::NotFound)?;

    let vault_key = &auth.vault_key;

    // For each Option<String> field:
    //   None   → don't update
    //   Some("") → clear (set to NULL for nullable fields)
    //   Some(s) → encrypt and set

    let title_enc: Option<String> = payload.title.as_ref()
        .map(|s| encrypt_str(vault_key, s.as_str()))
        .transpose()?;

    // username_enc: None=no change, Some(None)=clear, Some(Some(enc))=update
    let username_enc: Option<Option<String>> = match &payload.username {
        None => None,
        Some(s) if s.is_empty() => Some(None),
        Some(s) => Some(Some(encrypt_str(vault_key, s.as_str())?)),
    };

    let password_enc: Option<String> = payload.password.as_ref()
        .map(|s| encrypt_str(vault_key, s.as_str()))
        .transpose()?;

    let url_enc: Option<Option<String>> = match &payload.url {
        None => None,
        Some(s) if s.is_empty() => Some(None),
        Some(s) => Some(Some(encrypt_str(vault_key, s.as_str())?)),
    };

    let notes_enc: Option<Option<String>> = match &payload.notes {
        None => None,
        Some(s) if s.is_empty() => Some(None),
        Some(s) => Some(Some(encrypt_str(vault_key, s.as_str())?)),
    };

    db::update_vault_entry(
        &state.pool,
        &id,
        &auth.user.id,
        title_enc.as_deref(),
        username_enc.as_ref().map(|o: &Option<String>| o.as_deref()),
        password_enc.as_deref(),
        url_enc.as_ref().map(|o: &Option<String>| o.as_deref()),
        notes_enc.as_ref().map(|o: &Option<String>| o.as_deref()),
        payload.category.as_deref(),
        payload.is_favorite,
    )
    .await?;

    // Return the updated entry
    let updated_row = db::get_vault_entry(&state.pool, &id, &auth.user.id)
        .await?
        .ok_or(AppError::NotFound)?;

    let entry = VaultEntry {
        id: updated_row.id,
        title: decrypt_str(vault_key, &updated_row.title_enc)?,
        username: updated_row
            .username_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        password: decrypt_str(vault_key, &updated_row.password_enc)?,
        url: updated_row
            .url_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        notes: updated_row
            .notes_enc
            .as_deref()
            .map(|enc| decrypt_str(vault_key, enc))
            .transpose()?,
        category: updated_row.category,
        is_favorite: updated_row.is_favorite,
        created_at: updated_row.created_at,
        updated_at: updated_row.updated_at,
    };

    Ok(Json(entry))
}

/// DELETE /api/vault/entries/:id
pub async fn delete_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let deleted = db::delete_vault_entry(&state.pool, &id, &auth.user.id).await?;
    if deleted {
        Ok(Json(json!({ "message": "Entry deleted" })))
    } else {
        Err(AppError::NotFound)
    }
}

/// POST /api/generate-password
pub async fn generate_password_handler(
    Json(payload): Json<GeneratePasswordRequest>,
) -> Json<GeneratePasswordResponse> {
    let length = payload.length.unwrap_or(16).clamp(8, 128);
    let uppercase = payload.uppercase.unwrap_or(true);
    let digits = payload.digits.unwrap_or(true);
    let symbols = payload.symbols.unwrap_or(true);

    let password = generate_password(length, uppercase, digits, symbols);
    Json(GeneratePasswordResponse { password })
}
