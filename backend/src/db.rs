use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::models::{Session, User, VaultEntryRow};

// ── Users ────────────────────────────────────────────────────────────────────

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
    encrypted_vault_key: &str,
) -> AppResult<User> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, encrypted_vault_key, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(username)
    .bind(password_hash)
    .bind(encrypted_vault_key)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(User {
        id,
        username: username.to_string(),
        password_hash: password_hash.to_string(),
        encrypted_vault_key: encrypted_vault_key.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> AppResult<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash, encrypted_vault_key, created_at, updated_at
         FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: r.get("id"),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        encrypted_vault_key: r.get("encrypted_vault_key"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> AppResult<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash, encrypted_vault_key, created_at, updated_at
         FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: r.get("id"),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        encrypted_vault_key: r.get("encrypted_vault_key"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

// ── Sessions ─────────────────────────────────────────────────────────────────

pub async fn create_session(
    pool: &SqlitePool,
    user_id: &str,
    encrypted_session_key: &str,
) -> AppResult<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::hours(24);
    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let expires_str = expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, encrypted_session_key, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(encrypted_session_key)
    .bind(&expires_str)
    .bind(&now_str)
    .execute(pool)
    .await?;

    Ok(Session {
        id,
        user_id: user_id.to_string(),
        encrypted_session_key: encrypted_session_key.to_string(),
        expires_at: expires_str,
        created_at: now_str,
    })
}

pub async fn get_session(pool: &SqlitePool, session_id: &str) -> AppResult<Option<Session>> {
    let row = sqlx::query(
        "SELECT id, user_id, encrypted_session_key, expires_at, created_at
         FROM sessions WHERE id = ? AND expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Session {
        id: r.get("id"),
        user_id: r.get("user_id"),
        encrypted_session_key: r.get("encrypted_session_key"),
        expires_at: r.get("expires_at"),
        created_at: r.get("created_at"),
    }))
}

pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM sessions WHERE expires_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── VaultEntries ─────────────────────────────────────────────────────────────

pub async fn create_vault_entry(
    pool: &SqlitePool,
    user_id: &str,
    title_enc: &str,
    username_enc: Option<&str>,
    password_enc: &str,
    url_enc: Option<&str>,
    notes_enc: Option<&str>,
    category: &str,
    is_favorite: bool,
) -> AppResult<VaultEntryRow> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO vault_entries
         (id, user_id, title_enc, username_enc, password_enc, url_enc, notes_enc, category, is_favorite, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(title_enc)
    .bind(username_enc)
    .bind(password_enc)
    .bind(url_enc)
    .bind(notes_enc)
    .bind(category)
    .bind(is_favorite as i64)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(VaultEntryRow {
        id,
        user_id: user_id.to_string(),
        title_enc: title_enc.to_string(),
        username_enc: username_enc.map(|s| s.to_string()),
        password_enc: password_enc.to_string(),
        url_enc: url_enc.map(|s| s.to_string()),
        notes_enc: notes_enc.map(|s| s.to_string()),
        category: category.to_string(),
        is_favorite,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub async fn list_vault_entries(
    pool: &SqlitePool,
    user_id: &str,
) -> AppResult<Vec<VaultEntryRow>> {
    let rows = sqlx::query(
        "SELECT id, user_id, title_enc, username_enc, password_enc, url_enc, notes_enc,
                category, is_favorite, created_at, updated_at
         FROM vault_entries WHERE user_id = ? ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| VaultEntryRow {
            id: r.get("id"),
            user_id: r.get("user_id"),
            title_enc: r.get("title_enc"),
            username_enc: r.get("username_enc"),
            password_enc: r.get("password_enc"),
            url_enc: r.get("url_enc"),
            notes_enc: r.get("notes_enc"),
            category: r.get("category"),
            is_favorite: r.get::<i64, _>("is_favorite") != 0,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect())
}

pub async fn get_vault_entry(
    pool: &SqlitePool,
    entry_id: &str,
    user_id: &str,
) -> AppResult<Option<VaultEntryRow>> {
    let row = sqlx::query(
        "SELECT id, user_id, title_enc, username_enc, password_enc, url_enc, notes_enc,
                category, is_favorite, created_at, updated_at
         FROM vault_entries WHERE id = ? AND user_id = ?",
    )
    .bind(entry_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| VaultEntryRow {
        id: r.get("id"),
        user_id: r.get("user_id"),
        title_enc: r.get("title_enc"),
        username_enc: r.get("username_enc"),
        password_enc: r.get("password_enc"),
        url_enc: r.get("url_enc"),
        notes_enc: r.get("notes_enc"),
        category: r.get("category"),
        is_favorite: r.get::<i64, _>("is_favorite") != 0,
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn update_vault_entry(
    pool: &SqlitePool,
    entry_id: &str,
    user_id: &str,
    title_enc: Option<&str>,
    username_enc: Option<Option<&str>>,
    password_enc: Option<&str>,
    url_enc: Option<Option<&str>>,
    notes_enc: Option<Option<&str>>,
    category: Option<&str>,
    is_favorite: Option<bool>,
) -> AppResult<bool> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Build dynamic UPDATE
    let mut sets: Vec<String> = vec!["updated_at = ?".to_string()];

    if title_enc.is_some() { sets.push("title_enc = ?".to_string()); }
    if username_enc.is_some() { sets.push("username_enc = ?".to_string()); }
    if password_enc.is_some() { sets.push("password_enc = ?".to_string()); }
    if url_enc.is_some() { sets.push("url_enc = ?".to_string()); }
    if notes_enc.is_some() { sets.push("notes_enc = ?".to_string()); }
    if category.is_some() { sets.push("category = ?".to_string()); }
    if is_favorite.is_some() { sets.push("is_favorite = ?".to_string()); }

    let sql = format!(
        "UPDATE vault_entries SET {} WHERE id = ? AND user_id = ?",
        sets.join(", ")
    );

    let mut q = sqlx::query(&sql).bind(&now);
    if let Some(v) = title_enc { q = q.bind(v); }
    if let Some(v) = username_enc { q = q.bind(v); }
    if let Some(v) = password_enc { q = q.bind(v); }
    if let Some(v) = url_enc { q = q.bind(v); }
    if let Some(v) = notes_enc { q = q.bind(v); }
    if let Some(v) = category { q = q.bind(v); }
    if let Some(v) = is_favorite { q = q.bind(v as i64); }
    q = q.bind(entry_id).bind(user_id);

    let result = q.execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_vault_entry(
    pool: &SqlitePool,
    entry_id: &str,
    user_id: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM vault_entries WHERE id = ? AND user_id = ?",
    )
    .bind(entry_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
