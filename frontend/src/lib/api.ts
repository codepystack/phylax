const BASE_URL = "/api";

export interface VaultEntry {
	id: string;
	title: string;
	username?: string | null;
	password: string;
	url?: string | null;
	notes?: string | null;
	category: string;
	is_favorite: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateEntryPayload {
	title: string;
	username?: string;
	password: string;
	url?: string;
	notes?: string;
	category?: string;
	is_favorite?: boolean;
}

export interface UpdateEntryPayload {
	title?: string;
	username?: string;
	password?: string;
	url?: string;
	notes?: string;
	category?: string;
	is_favorite?: boolean;
}

async function apiFetch<T>(
	path: string,
	options: RequestInit = {},
	token?: string | null
): Promise<T> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
	};
	if (token) {
		headers["Authorization"] = `Bearer ${token}`;
	}

	const res = await fetch(`${BASE_URL}${path}`, {
		...options,
		headers: { ...headers, ...(options.headers as Record<string, string> || {}) },
	});

	if (!res.ok) {
		const err = await res.json().catch(() => ({ error: "Unknown error" }));
		throw new Error(err.error || `HTTP ${res.status}`);
	}

	return res.json();
}

// ── Auth ─────────────────────────────────────────────────────────────────────

export async function register(username: string, password: string) {
	return apiFetch<{ session_token: string; user_id: string; username: string }>(
		"/auth/register",
		{ method: "POST", body: JSON.stringify({ username, password }) }
	);
}

export async function login(username: string, password: string) {
	return apiFetch<{ session_token: string; user_id: string; username: string }>(
		"/auth/login",
		{ method: "POST", body: JSON.stringify({ username, password }) }
	);
}

export async function logout(token: string) {
	return apiFetch<{ message: string }>(
		"/auth/logout",
		{ method: "POST" },
		token
	);
}

// ── Vault ─────────────────────────────────────────────────────────────────────

export async function listEntries(token: string): Promise<VaultEntry[]> {
	return apiFetch<VaultEntry[]>("/vault/entries", {}, token);
}

export async function getEntry(token: string, id: string): Promise<VaultEntry> {
	return apiFetch<VaultEntry>(`/vault/entries/${id}`, {}, token);
}

export async function createEntry(token: string, payload: CreateEntryPayload): Promise<VaultEntry> {
	return apiFetch<VaultEntry>(
		"/vault/entries",
		{ method: "POST", body: JSON.stringify(payload) },
		token
	);
}

export async function updateEntry(
	token: string,
	id: string,
	payload: UpdateEntryPayload
): Promise<VaultEntry> {
	return apiFetch<VaultEntry>(
		`/vault/entries/${id}`,
		{ method: "PUT", body: JSON.stringify(payload) },
		token
	);
}

export async function deleteEntry(token: string, id: string): Promise<{ message: string }> {
	return apiFetch<{ message: string }>(
		`/vault/entries/${id}`,
		{ method: "DELETE" },
		token
	);
}

// ── Password Generator ────────────────────────────────────────────────────────

export async function generatePassword(options: {
	length?: number;
	uppercase?: boolean;
	digits?: boolean;
	symbols?: boolean;
}): Promise<{ password: string }> {
	return apiFetch<{ password: string }>(
		"/generate-password",
		{ method: "POST", body: JSON.stringify(options) }
	);
}
