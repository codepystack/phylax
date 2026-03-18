import { writable } from "svelte/store";

export type ToastVariant = "default" | "destructive" | "success";

export interface Toast {
	id: string;
	message: string;
	variant: ToastVariant;
}

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	function show(message: string, variant: ToastVariant = "default", duration = 3000) {
		const id = Math.random().toString(36).slice(2);
		update((toasts) => [...toasts, { id, message, variant }]);
		setTimeout(() => {
			update((toasts) => toasts.filter((t) => t.id !== id));
		}, duration);
	}

	return {
		subscribe,
		success: (message: string) => show(message, "success"),
		error: (message: string) => show(message, "destructive"),
		info: (message: string) => show(message, "default"),
	};
}

export const toast = createToastStore();
