<script lang="ts">
	import { cn } from "$lib/utils";
	let { class: className = "" } = $props();
</script>
<hr class={cn("shrink-0 bg-border border-none h-px w-full", className)} />
