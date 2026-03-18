<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth';
	import { toast } from '$lib/stores/toast';
	import {
		listEntries,
		createEntry,
		updateEntry,
		deleteEntry,
		generatePassword,
		logout,
		type VaultEntry,
		type CreateEntryPayload
	} from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import {
		Dialog,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogFooter
	} from '$lib/components/ui/dialog';
	import { Separator } from '$lib/components/ui/separator';

	// ── State ───────────────────────────────────────────────────────────────────

	let entries: VaultEntry[] = $state([]);
	let loading = $state(true);
	let searchQuery = $state('');
	let selectedCategory = $state('all');

	// Entry form dialog
	let showDialog = $state(false);
	let editingEntry: VaultEntry | null = $state(null);
	let formLoading = $state(false);

	// Password generator
	let showGenerator = $state(false);
	let genLength = $state(16);
	let genUppercase = $state(true);
	let genDigits = $state(true);
	let genSymbols = $state(true);
	let generatedPassword = $state('');
	let genLoading = $state(false);

	// Copy tooltip tracking
	let copiedId = $state<string | null>(null);

	// Detail view
	let showDetail = $state(false);
	let detailEntry: VaultEntry | null = $state(null);
	let detailPasswordVisible = $state(false);

	// Form fields
	let formTitle = $state('');
	let formUsername = $state('');
	let formPassword = $state('');
	let formUrl = $state('');
	let formNotes = $state('');
	let formCategory = $state('login');
	let formFavorite = $state(false);
	let formPasswordVisible = $state(false);

	// ── Auth guard ───────────────────────────────────────────────────────────────

	let token = $derived($auth.token);
	let username = $derived($auth.username);

	onMount(async () => {
		if (!token) {
			goto('/');
			return;
		}
		await loadEntries();
	});

	// ── API helpers ──────────────────────────────────────────────────────────────

	async function loadEntries() {
		loading = true;
		try {
			entries = await listEntries(token!);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : 'Failed to load vault';
			toast.error(msg);
			if (msg === 'Unauthorized') {
				auth.logout();
				goto('/');
			}
		} finally {
			loading = false;
		}
	}

	async function handleLogout() {
		try {
			await logout(token!);
		} catch {
			// ignore
		}
		auth.logout();
		toast.info('Signed out');
		goto('/');
	}

	// ── Form ─────────────────────────────────────────────────────────────────────

	function openNew() {
		editingEntry = null;
		formTitle = '';
		formUsername = '';
		formPassword = '';
		formUrl = '';
		formNotes = '';
		formCategory = 'login';
		formFavorite = false;
		formPasswordVisible = false;
		generatedPassword = '';
		showDialog = true;
	}

	function openEdit(entry: VaultEntry) {
		editingEntry = entry;
		formTitle = entry.title;
		formUsername = entry.username ?? '';
		formPassword = entry.password;
		formUrl = entry.url ?? '';
		formNotes = entry.notes ?? '';
		formCategory = entry.category;
		formFavorite = entry.is_favorite;
		formPasswordVisible = false;
		generatedPassword = '';
		showDetail = false;
		showDialog = true;
	}

	async function handleSave(e: SubmitEvent) {
		e.preventDefault();
		if (!formTitle.trim() || !formPassword) return;

		formLoading = true;
		try {
			if (editingEntry) {
				const updated = await updateEntry(token!, editingEntry.id, {
					title: formTitle.trim(),
					username: formUsername || undefined,
					password: formPassword,
					url: formUrl || undefined,
					notes: formNotes || undefined,
					category: formCategory,
					is_favorite: formFavorite,
				});
				entries = entries.map((e) => (e.id === updated.id ? updated : e));
				toast.success('Entry updated');
			} else {
				const payload: CreateEntryPayload = {
					title: formTitle.trim(),
					password: formPassword,
					category: formCategory,
					is_favorite: formFavorite,
				};
				if (formUsername) payload.username = formUsername;
				if (formUrl) payload.url = formUrl;
				if (formNotes) payload.notes = formNotes;
				const created = await createEntry(token!, payload);
				entries = [created, ...entries];
				toast.success('Entry created');
			}
			showDialog = false;
		} catch (err: unknown) {
			toast.error(err instanceof Error ? err.message : 'Save failed');
		} finally {
			formLoading = false;
		}
	}

	async function handleDelete(entry: VaultEntry) {
		if (!confirm(`Delete "${entry.title}"? This cannot be undone.`)) return;
		try {
			await deleteEntry(token!, entry.id);
			entries = entries.filter((e) => e.id !== entry.id);
			if (showDetail && detailEntry?.id === entry.id) showDetail = false;
			toast.success('Entry deleted');
		} catch (err: unknown) {
			toast.error(err instanceof Error ? err.message : 'Delete failed');
		}
	}

	// ── Password generator ────────────────────────────────────────────────────────

	async function handleGenerate() {
		genLoading = true;
		try {
			const res = await generatePassword({
				length: genLength,
				uppercase: genUppercase,
				digits: genDigits,
				symbols: genSymbols,
			});
			generatedPassword = res.password;
		} catch (err: unknown) {
			toast.error(err instanceof Error ? err.message : 'Generator failed');
		} finally {
			genLoading = false;
		}
	}

	function useGeneratedPassword() {
		formPassword = generatedPassword;
		showGenerator = false;
	}

	// ── Copy to clipboard ─────────────────────────────────────────────────────────

	async function copyToClipboard(text: string, id: string) {
		try {
			await navigator.clipboard.writeText(text);
			copiedId = id;
			setTimeout(() => { copiedId = null; }, 2000);
		} catch {
			toast.error('Copy failed');
		}
	}

	// ── Detail view ───────────────────────────────────────────────────────────────

	function openDetail(entry: VaultEntry) {
		detailEntry = entry;
		detailPasswordVisible = false;
		showDetail = true;
	}

	// ── Filtering / search ────────────────────────────────────────────────────────

	let filteredEntries = $derived(
		entries.filter((e) => {
			const q = searchQuery.toLowerCase();
			const matchesSearch =
				!q ||
				e.title.toLowerCase().includes(q) ||
				(e.username ?? '').toLowerCase().includes(q) ||
				(e.url ?? '').toLowerCase().includes(q);
			const matchesCategory =
				selectedCategory === 'all' ||
				(selectedCategory === 'favorites' ? e.is_favorite : e.category === selectedCategory);
			return matchesSearch && matchesCategory;
		})
	);

	const categories = ['all', 'login', 'card', 'secure-note', 'favorites'];

	function categoryLabel(c: string) {
		return c === 'secure-note' ? 'Notes' : c.charAt(0).toUpperCase() + c.slice(1);
	}
</script>

<svelte:head>
	<title>Vault – Phylax</title>
</svelte:head>

<!-- ── App Shell ─────────────────────────────────────────────────────────── -->
<div class="min-h-screen bg-background flex flex-col">

	<!-- Header -->
	<header class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
		<div class="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				<svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
				</svg>
				<span class="font-semibold text-sm">Phylax</span>
			</div>

			<!-- Search -->
			<div class="flex-1 max-w-md">
				<Input
					type="search"
					placeholder="Search vault…"
					bind:value={searchQuery}
					class="h-8 text-sm"
				/>
			</div>

			<div class="flex items-center gap-2">
				<span class="text-sm text-muted-foreground hidden sm:inline">{username}</span>
				<Button onclick={openNew} size="sm">
					<svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
					</svg>
					New
				</Button>
				<Button onclick={handleLogout} variant="ghost" size="icon" title="Sign out">
					<svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
					</svg>
				</Button>
			</div>
		</div>
	</header>

	<!-- Main -->
	<main class="flex-1 max-w-6xl mx-auto w-full px-4 py-6">

		<!-- Category tabs -->
		<div class="flex gap-1 mb-6 flex-wrap">
			{#each categories as cat}
				<button
					class="px-3 py-1.5 text-sm rounded-md transition-colors
					{selectedCategory === cat
						? 'bg-primary text-primary-foreground'
						: 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
					onclick={() => (selectedCategory = cat)}
				>
					{categoryLabel(cat)}
					{#if cat === 'all'}
						<span class="ml-1 text-xs opacity-60">{entries.length}</span>
					{/if}
				</button>
			{/each}
		</div>

		<!-- Entries -->
		{#if loading}
			<div class="flex justify-center py-20">
				<svg class="animate-spin w-6 h-6 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else if filteredEntries.length === 0}
			<div class="text-center py-20 text-muted-foreground">
				{#if searchQuery || selectedCategory !== 'all'}
					<p class="text-lg font-medium">No results found</p>
					<p class="text-sm mt-1">Try a different search or category</p>
				{:else}
					<div class="mb-4">
						<svg xmlns="http://www.w3.org/2000/svg" class="w-12 h-12 mx-auto opacity-30" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
							<path stroke-linecap="round" stroke-linejoin="round" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
						</svg>
					</div>
					<p class="text-lg font-medium">Your vault is empty</p>
					<p class="text-sm mt-1">Add your first password to get started</p>
					<Button onclick={openNew} class="mt-4" size="sm">Add entry</Button>
				{/if}
			</div>
		{:else}
			<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
				{#each filteredEntries as entry (entry.id)}
					<div
						class="group relative rounded-lg border bg-card p-4 hover:shadow-md transition-all cursor-pointer"
						onclick={() => openDetail(entry)}
						onkeydown={(e) => e.key === 'Enter' && openDetail(entry)}
						role="button"
						tabindex="0"
						aria-label="View {entry.title}"
					>
						<!-- Icon + title -->
						<div class="flex items-start justify-between gap-2 mb-2">
							<div class="flex items-center gap-2 min-w-0">
								<div class="w-8 h-8 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
									{#if entry.category === 'card'}
										<svg class="w-4 h-4 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
										</svg>
									{:else if entry.category === 'secure-note'}
										<svg class="w-4 h-4 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
										</svg>
									{:else}
										<svg class="w-4 h-4 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
										</svg>
									{/if}
								</div>
								<div class="min-w-0">
									<p class="font-medium text-sm truncate">{entry.title}</p>
									{#if entry.username}
										<p class="text-xs text-muted-foreground truncate">{entry.username}</p>
									{/if}
								</div>
							</div>
							{#if entry.is_favorite}
								<svg class="w-4 h-4 text-yellow-500 shrink-0" viewBox="0 0 20 20" fill="currentColor">
									<path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
								</svg>
							{/if}
						</div>

						<!-- URL -->
						{#if entry.url}
							<p class="text-xs text-muted-foreground truncate mb-2">{entry.url}</p>
						{/if}

						<!-- Actions (visible on hover) -->
						<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
						<div class="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="group" aria-label="Entry actions">
							<button
								class="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
								title="Copy password"
								onclick={() => copyToClipboard(entry.password, entry.id + '-pwd')}
								aria-label="Copy password for {entry.title}"
							>
								{#if copiedId === entry.id + '-pwd'}
									<svg class="w-3.5 h-3.5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
									</svg>
								{:else}
									<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
									</svg>
								{/if}
							</button>
							<button
								class="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
								title="Edit"
								onclick={() => openEdit(entry)}
								aria-label="Edit {entry.title}"
							>
								<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
								</svg>
							</button>
							<button
								class="p-1 rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
								title="Delete"
								onclick={() => handleDelete(entry)}
								aria-label="Delete {entry.title}"
							>
								<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
								</svg>
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</main>
</div>

<!-- ── Entry Detail Dialog ────────────────────────────────────────────────── -->
<Dialog open={showDetail} onclose={() => (showDetail = false)}>
	{#if detailEntry}
		<DialogHeader>
			<div class="flex items-center justify-between">
				<DialogTitle>{detailEntry.title}</DialogTitle>
				<button
					class="rounded-md p-1 hover:bg-accent transition-colors"
					onclick={() => (showDetail = false)}
					aria-label="Close"
				>
					<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		</DialogHeader>

		<div class="space-y-4 mt-2">
			{#if detailEntry.username}
				<div>
					<p class="text-xs font-medium text-muted-foreground mb-1">Username</p>
					<div class="flex items-center justify-between gap-2 p-2 rounded-md bg-muted">
						<span class="text-sm font-mono">{detailEntry.username}</span>
						<button
							class="text-muted-foreground hover:text-foreground"
							onclick={() => copyToClipboard(detailEntry!.username!, 'detail-user')}
							aria-label="Copy username"
						>
							{#if copiedId === 'detail-user'}
								<svg class="w-4 h-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
								</svg>
							{:else}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
								</svg>
							{/if}
						</button>
					</div>
				</div>
			{/if}

			<div>
				<p class="text-xs font-medium text-muted-foreground mb-1">Password</p>
				<div class="flex items-center justify-between gap-2 p-2 rounded-md bg-muted">
					<span class="text-sm font-mono flex-1 overflow-hidden">
						{detailPasswordVisible ? detailEntry.password : '•'.repeat(Math.min(detailEntry.password.length, 20))}
					</span>
					<div class="flex items-center gap-1 shrink-0">
						<button
							class="text-muted-foreground hover:text-foreground"
							onclick={() => (detailPasswordVisible = !detailPasswordVisible)}
							aria-label={detailPasswordVisible ? 'Hide password' : 'Show password'}
						>
							{#if detailPasswordVisible}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
								</svg>
							{:else}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
									<path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
								</svg>
							{/if}
						</button>
						<button
							class="text-muted-foreground hover:text-foreground"
							onclick={() => copyToClipboard(detailEntry!.password, 'detail-pwd')}
							aria-label="Copy password"
						>
							{#if copiedId === 'detail-pwd'}
								<svg class="w-4 h-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
								</svg>
							{:else}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
								</svg>
							{/if}
						</button>
					</div>
				</div>
			</div>

			{#if detailEntry.url}
				<div>
					<p class="text-xs font-medium text-muted-foreground mb-1">URL</p>
					<div class="flex items-center gap-2 p-2 rounded-md bg-muted">
						<a
							href={detailEntry.url}
							target="_blank"
							rel="noopener noreferrer"
							class="text-sm text-primary hover:underline truncate flex-1"
							onclick={(e) => e.stopPropagation()}
						>{detailEntry.url}</a>
					</div>
				</div>
			{/if}

			{#if detailEntry.notes}
				<div>
					<p class="text-xs font-medium text-muted-foreground mb-1">Notes</p>
					<p class="text-sm p-2 rounded-md bg-muted whitespace-pre-wrap">{detailEntry.notes}</p>
				</div>
			{/if}

			<div class="flex items-center gap-2">
				<Badge variant="secondary">{detailEntry.category}</Badge>
				{#if detailEntry.is_favorite}
					<Badge variant="outline">⭐ Favorite</Badge>
				{/if}
			</div>
		</div>

		<div class="flex justify-end gap-2 mt-4">
			<Button variant="outline" onclick={() => openEdit(detailEntry!)}>Edit</Button>
			<Button variant="destructive" onclick={() => handleDelete(detailEntry!)}>Delete</Button>
		</div>
	{/if}
</Dialog>

<!-- ── Create / Edit Dialog ───────────────────────────────────────────────── -->
<Dialog open={showDialog} onclose={() => (showDialog = false)} class="max-w-lg">
	<DialogHeader>
		<div class="flex items-center justify-between">
			<DialogTitle>{editingEntry ? 'Edit entry' : 'New entry'}</DialogTitle>
			<button
				class="rounded-md p-1 hover:bg-accent transition-colors"
				onclick={() => (showDialog = false)}
				aria-label="Close"
			>
				<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		</div>
	</DialogHeader>

	<form onsubmit={handleSave} class="mt-4">
		<div class="space-y-4">
			<!-- Title -->
			<div class="space-y-1.5">
				<Label for="form-title">Title <span class="text-destructive">*</span></Label>
				<Input id="form-title" bind:value={formTitle} placeholder="e.g. GitHub" required />
			</div>

			<!-- Category -->
			<div class="space-y-1.5">
				<Label for="form-category">Category</Label>
				<select
					id="form-category"
					bind:value={formCategory}
					class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
				>
					<option value="login">Login</option>
					<option value="card">Card</option>
					<option value="secure-note">Secure Note</option>
					<option value="other">Other</option>
				</select>
			</div>

			<!-- Username -->
			<div class="space-y-1.5">
				<Label for="form-username">Username / Email</Label>
				<Input id="form-username" bind:value={formUsername} placeholder="user@example.com" autocomplete="off" />
			</div>

			<!-- Password -->
			<div class="space-y-1.5">
				<div class="flex items-center justify-between">
					<Label for="form-password">Password <span class="text-destructive">*</span></Label>
					<button
						type="button"
						class="text-xs text-primary hover:underline"
						onclick={() => (showGenerator = !showGenerator)}
					>
						{showGenerator ? 'Hide generator' : 'Generate'}
					</button>
				</div>
				<div class="flex gap-2">
					<div class="relative flex-1">
						<Input
							id="form-password"
							type={formPasswordVisible ? 'text' : 'password'}
							bind:value={formPassword}
							placeholder="Enter password"
							required
							autocomplete="new-password"
							class="pr-10"
						/>
						<button
							type="button"
							class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
							onclick={() => (formPasswordVisible = !formPasswordVisible)}
							aria-label={formPasswordVisible ? 'Hide password' : 'Show password'}
						>
							{#if formPasswordVisible}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
								</svg>
							{:else}
								<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
									<path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
								</svg>
							{/if}
						</button>
					</div>
					<button
						type="button"
						class="p-2 rounded-md border hover:bg-accent transition-colors"
						onclick={() => copyToClipboard(formPassword, 'form-pwd')}
						aria-label="Copy password"
						title="Copy"
					>
						<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
						</svg>
					</button>
				</div>

				<!-- Password Generator -->
				{#if showGenerator}
					<div class="rounded-md border p-3 bg-muted/50 space-y-3 mt-2">
						<div class="flex items-center gap-3">
							<Label for="gen-length" class="shrink-0">Length: {genLength}</Label>
							<input
								id="gen-length"
								type="range"
								min="8"
								max="64"
								bind:value={genLength}
								class="flex-1"
							/>
						</div>
						<div class="flex flex-wrap gap-3 text-sm">
							<label class="flex items-center gap-1.5 cursor-pointer">
								<input type="checkbox" bind:checked={genUppercase} class="rounded" />
								Uppercase
							</label>
							<label class="flex items-center gap-1.5 cursor-pointer">
								<input type="checkbox" bind:checked={genDigits} class="rounded" />
								Digits
							</label>
							<label class="flex items-center gap-1.5 cursor-pointer">
								<input type="checkbox" bind:checked={genSymbols} class="rounded" />
								Symbols
							</label>
						</div>

						{#if generatedPassword}
							<div class="flex items-center gap-2 p-2 rounded-md bg-background border">
								<code class="text-xs flex-1 font-mono overflow-x-auto">{generatedPassword}</code>
								<button
									type="button"
									class="shrink-0 text-muted-foreground hover:text-foreground"
									onclick={() => copyToClipboard(generatedPassword, 'gen-copy')}
									aria-label="Copy generated password"
								>
									<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
									</svg>
								</button>
							</div>
						{/if}

						<div class="flex gap-2">
							<Button
								type="button"
								variant="outline"
								size="sm"
								onclick={handleGenerate}
								disabled={genLoading}
							>
								{genLoading ? 'Generating…' : 'Generate'}
							</Button>
							{#if generatedPassword}
								<Button type="button" size="sm" onclick={useGeneratedPassword}>Use this</Button>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<!-- URL -->
			<div class="space-y-1.5">
				<Label for="form-url">URL</Label>
				<Input id="form-url" type="url" bind:value={formUrl} placeholder="https://example.com" autocomplete="off" />
			</div>

			<!-- Notes -->
			<div class="space-y-1.5">
				<Label for="form-notes">Notes</Label>
				<textarea
					id="form-notes"
					bind:value={formNotes}
					placeholder="Optional notes…"
					rows="3"
					class="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none"
				></textarea>
			</div>

			<!-- Favorite -->
			<label class="flex items-center gap-2 cursor-pointer">
				<input type="checkbox" bind:checked={formFavorite} class="rounded" />
				<span class="text-sm">Mark as favorite</span>
			</label>
		</div>

		<div class="flex justify-end gap-2 mt-6">
			<Button type="button" variant="outline" onclick={() => (showDialog = false)}>Cancel</Button>
			<Button type="submit" disabled={formLoading}>
				{formLoading ? 'Saving…' : editingEntry ? 'Save changes' : 'Add entry'}
			</Button>
		</div>
	</form>
</Dialog>
