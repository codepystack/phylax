use serde::{Deserialize, Serialize};

// ── User ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub encrypted_vault_key: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── Session ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    /// AES-256-GCM encrypted copy of the vault key for this session.
    /// The session key is stored in memory only and never persisted directly.
    pub encrypted_session_key: String,
    pub expires_at: String,
    pub created_at: String,
}

// ── VaultEntry ──────────────────────────────────────────────────────────────

/// Raw DB row – all sensitive columns are still encrypted.
#[derive(Debug, Clone)]
pub struct VaultEntryRow {
    pub id: String,
    pub user_id: String,
    pub title_enc: String,
    pub username_enc: Option<String>,
    pub password_enc: String,
    pub url_enc: Option<String>,
    pub notes_enc: Option<String>,
    pub category: String,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Decrypted vault entry sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: String,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ── API request/response types ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub session_token: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub title: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntryRequest {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GeneratePasswordRequest {
    pub length: Option<usize>,
    pub uppercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GeneratePasswordResponse {
    pub password: String,
}
