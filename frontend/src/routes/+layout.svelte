<script lang="ts">
	import './layout.css';
	import { toast } from '$lib/stores/toast';

	let { children } = $props();
</script>

{@render children()}

<!-- Toast notifications -->
<div class="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 max-w-sm w-full pointer-events-none">
	{#each $toast as t (t.id)}
		<div
			class="pointer-events-auto rounded-lg border p-4 shadow-lg text-sm font-medium transition-all animate-in slide-in-from-right-full
			{t.variant === 'destructive' ? 'bg-destructive text-destructive-foreground border-destructive' :
			 t.variant === 'success' ? 'bg-green-600 text-white border-green-700' :
			 'bg-background text-foreground border-border'}"
			role="status"
			aria-live="polite"
		>
			{t.message}
		</div>
	{/each}
</div>

