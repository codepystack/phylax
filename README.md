# Phylax — Hardened Password Manager

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A self-hosted password manager with a Rust backend and a clean SvelteKit + shadcn-svelte UI, accessible from any device.

## Screenshots

| Login | Vault | New Entry |
|---|---|---|
| ![Login page](https://github.com/user-attachments/assets/f147fe65-f12b-4f17-8a8d-522cf85d75de) | ![Vault dashboard](https://github.com/user-attachments/assets/d5fedab6-868e-4b5c-be19-1072cd69571a) | ![New entry dialog](https://github.com/user-attachments/assets/e5d92ec9-1503-4360-8ee9-4aa1a45044f4) |

## Security Model

| Layer | Mechanism |
|---|---|
| Master password hashing | **Argon2id** (64 MiB memory, 3 iterations, 4 parallelism) |
| Vault key derivation | **Argon2id KDF** from master password + user-id salt |
| Vault entry encryption | **AES-256-GCM** with a fresh 96-bit random nonce per field |
| Session key wrapping | AES-256-GCM with a server-side secret (set via `SERVER_SECRET` env) |
| Auth rate limiting | 10 requests per minute (in-memory governor) |
| Sensitive key material | Zeroize-on-drop wrappers prevent keys leaking in heap memory |

The vault key is **never stored in plaintext**. It exists only in memory for the duration of a session, wrapped by the server secret. All database columns containing passwords, usernames, URLs, or notes are individually encrypted.

## Tech Stack

- **Backend**: Rust · Axum 0.7 · SQLite (sqlx) · Argon2 · AES-256-GCM · Tower-HTTP
- **Frontend**: SvelteKit 2 · Svelte 5 · shadcn-svelte · Tailwind CSS v4 · TypeScript

## Project Structure

```
phylax/
├── backend/          # Rust Axum server
│   ├── Cargo.toml
│   ├── .env.example
│   ├── migrations/   # SQLite schema
│   └── src/
│       ├── main.rs       # Server bootstrap + router
│       ├── auth.rs       # Auth middleware + server secret
│       ├── crypto.rs     # Argon2id, AES-256-GCM, password generator
│       ├── db.rs         # Database helpers
│       ├── error.rs      # Typed error → HTTP response mapping
│       ├── models.rs     # Domain types + API request/response types
│       ├── state.rs      # AppState (DB pool + rate limiter)
│       └── routes/
│           ├── auth_routes.rs   # POST /api/auth/{register,login,logout}
│           └── vault_routes.rs  # CRUD /api/vault/entries + POST /api/generate-password
└── frontend/         # SvelteKit application
    ├── vite.config.ts      # Dev proxy → backend :8080
    ├── src/
    │   ├── lib/
    │   │   ├── api.ts            # Typed fetch wrappers for all API endpoints
    │   │   ├── utils.ts          # cn() Tailwind class merger
    │   │   ├── stores/
    │   │   │   ├── auth.ts       # Auth state (localStorage-persisted token)
    │   │   │   └── toast.ts      # Lightweight toast notifications
    │   │   └── components/ui/    # shadcn-svelte components
    │   │       ├── button/
    │   │       ├── input/
    │   │       ├── label/
    │   │       ├── card/
    │   │       ├── badge/
    │   │       ├── dialog/
    │   │       ├── separator/
    │   │       └── alert/
    │   └── routes/
    │       ├── +page.svelte      # Login / Register
    │       └── vault/
    │           └── +page.svelte  # Vault dashboard (list, create, edit, delete)
    └── ...
```

## API Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/api/auth/register` | — | Create account |
| POST | `/api/auth/login` | — | Get session token |
| POST | `/api/auth/logout` | ✓ | Invalidate session |
| GET | `/api/vault/entries` | ✓ | List decrypted entries |
| POST | `/api/vault/entries` | ✓ | Create entry |
| GET | `/api/vault/entries/:id` | ✓ | Get single entry |
| PUT | `/api/vault/entries/:id` | ✓ | Update entry |
| DELETE | `/api/vault/entries/:id` | ✓ | Delete entry |
| POST | `/api/generate-password` | — | Generate secure password |

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- Node.js 20+

### Backend

```bash
cd backend
cp .env.example .env
# Edit .env and set a strong SERVER_SECRET

cargo run
# Server starts on http://localhost:8080
```

### Frontend (development)

```bash
cd frontend
npm install
npm run dev
# Dev server on http://localhost:5173, proxies /api → :8080
```

### Production Build

```bash
# 1. Build the frontend
cd frontend && npm run build

# 2. Build and run the backend (which serves frontend/build/)
cd backend && cargo build --release
DATABASE_URL=sqlite://phylax.db SERVER_SECRET=<your-secret> ./target/release/phylax-server
```

## Features

- 🔐 **End-to-end encryption** — every credential field encrypted with AES-256-GCM before storage
- 🔑 **Secure master password** — never stored; Argon2id hash + derived vault key
- 🎲 **Password generator** — configurable length, charset (uppercase, digits, symbols)
- 🗂️ **Categories** — Login, Card, Secure Note, Other; filter by category or favourites
- 🔍 **Search** — live search across title, username, and URL
- 📋 **Copy to clipboard** — one-click copy for username and password
- 👁️ **Show/hide password** — toggle visibility in detail view and entry form
- 📱 **Responsive** — mobile-first grid layout, accessible from any device
- ⚡ **Self-hosted** — single binary + SQLite, no cloud dependency

## Running Tests

```bash
cd backend && cargo test
```

All cryptographic primitives (Argon2id, AES-256-GCM, key derivation, password generation) are covered by unit tests.

## Contributing

Contributions are welcome! Feel free to open issues for bug reports, feature requests, or security concerns. To contribute code:

1. Fork the repository and create a feature branch.
2. Make your changes with clear commit messages.
3. Ensure `cargo test` passes for any backend changes.
4. Open a pull request describing what you changed and why.

Please be respectful and constructive in all interactions.

## License

This project is licensed under the [MIT License](LICENSE).
