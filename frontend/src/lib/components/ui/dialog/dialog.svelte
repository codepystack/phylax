<script lang="ts">
	import { cn } from "$lib/utils";

	interface Props {
		open?: boolean;
		onclose?: () => void;
		class?: string;
		children?: import("svelte").Snippet;
	}

	let { open = false, onclose, class: className = "", children }: Props = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose?.();
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_interactive_supports_focus -->
	<div
		class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4"
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		onclick={handleBackdropClick}
	>
		<div
			class={cn(
				"relative z-50 grid w-full max-w-lg gap-4 rounded-xl border bg-background p-6 shadow-lg",
				className
			)}
		>
			{@render children?.()}
		</div>
	</div>
{/if}
