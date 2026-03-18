-- Enable WAL mode for better concurrent read performance
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY NOT NULL,
    username    TEXT NOT NULL UNIQUE,
    -- Argon2id hash of the master password
    password_hash  TEXT NOT NULL,
    -- AES-256-GCM encrypted vault key (nonce prepended, base64 encoded)
    -- The vault key itself is derived from the master password
    encrypted_vault_key TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- AES-256-GCM encrypted session vault key (for in-memory decryption)
    -- nonce prepended, base64 encoded
    encrypted_session_key TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Vault entries table
CREATE TABLE IF NOT EXISTS vault_entries (
    id          TEXT PRIMARY KEY NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- All sensitive fields are AES-256-GCM encrypted with the vault key
    -- Each field stores: base64(nonce || ciphertext)
    title_enc   TEXT NOT NULL,
    username_enc TEXT,
    password_enc TEXT NOT NULL,
    url_enc     TEXT,
    notes_enc   TEXT,
    category    TEXT NOT NULL DEFAULT 'login',
    is_favorite INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_vault_entries_user_id ON vault_entries(user_id);
CREATE INDEX IF NOT EXISTS idx_vault_entries_category ON vault_entries(category);
