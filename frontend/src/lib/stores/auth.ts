import { writable } from "svelte/store";
import { browser } from "$app/environment";

export interface AuthState {
	token: string | null;
	userId: string | null;
	username: string | null;
}

function createAuthStore() {
	const initial: AuthState =
		browser
			? {
					token: localStorage.getItem("phylax_token"),
					userId: localStorage.getItem("phylax_user_id"),
					username: localStorage.getItem("phylax_username"),
				}
			: { token: null, userId: null, username: null };

	const { subscribe, set, update } = writable<AuthState>(initial);

	return {
		subscribe,
		login(token: string, userId: string, username: string) {
			if (browser) {
				localStorage.setItem("phylax_token", token);
				localStorage.setItem("phylax_user_id", userId);
				localStorage.setItem("phylax_username", username);
			}
			set({ token, userId, username });
		},
		logout() {
			if (browser) {
				localStorage.removeItem("phylax_token");
				localStorage.removeItem("phylax_user_id");
				localStorage.removeItem("phylax_username");
			}
			set({ token: null, userId: null, username: null });
		},
	};
}

export const auth = createAuthStore();
