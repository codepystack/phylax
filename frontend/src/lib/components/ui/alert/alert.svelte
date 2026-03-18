<script lang="ts">
	import { cn } from "$lib/utils";
	type Variant = "default" | "destructive" | "success";
	interface Props {
		variant?: Variant;
		class?: string;
		children?: import("svelte").Snippet;
	}
	let { variant = "default", class: className = "", children }: Props = $props();

	const variantClasses: Record<Variant, string> = {
		default: "bg-background text-foreground",
		destructive: "border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive",
		success: "border-green-500/50 text-green-700 dark:text-green-400",
	};
</script>

<div
	class={cn(
		"relative w-full rounded-lg border p-4 [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground",
		variantClasses[variant],
		className
	)}
	role="alert"
>
	{@render children?.()}
</div>
