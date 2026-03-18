<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth';
	import { toast } from '$lib/stores/toast';
	import { login, register } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '$lib/components/ui/card';

	let mode: 'login' | 'register' = $state('login');
	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!username.trim() || !password) return;

		if (mode === 'register' && password !== confirmPassword) {
			toast.error('Passwords do not match');
			return;
		}
		if (mode === 'register' && password.length < 8) {
			toast.error('Password must be at least 8 characters');
			return;
		}

		loading = true;
		try {
			const res = mode === 'login'
				? await login(username.trim(), password)
				: await register(username.trim(), password);

			auth.login(res.session_token, res.user_id, res.username);
			toast.success(mode === 'login' ? 'Welcome back!' : 'Account created!');
			goto('/vault');
		} catch (err: unknown) {
			const message = err instanceof Error ? err.message : 'Something went wrong';
			toast.error(message);
		} finally {
			loading = false;
		}
	}

	function toggleMode() {
		mode = mode === 'login' ? 'register' : 'login';
		username = '';
		password = '';
		confirmPassword = '';
	}
</script>

<svelte:head>
	<title>Phylax – Password Manager</title>
</svelte:head>

<main class="min-h-screen flex items-center justify-center bg-background px-4">
	<div class="w-full max-w-sm">
		<!-- Logo / Brand -->
		<div class="text-center mb-8">
			<div class="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary mb-4">
				<svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 text-primary-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
				</svg>
			</div>
			<h1 class="text-2xl font-bold tracking-tight">Phylax</h1>
			<p class="text-sm text-muted-foreground mt-1">Your hardened password vault</p>
		</div>

		<Card>
			<CardHeader>
				<CardTitle>{mode === 'login' ? 'Sign in' : 'Create account'}</CardTitle>
				<CardDescription>
					{mode === 'login'
						? 'Enter your credentials to access your vault'
						: 'Create a new account to get started'}
				</CardDescription>
			</CardHeader>
			<form onsubmit={handleSubmit}>
				<CardContent class="space-y-4">
					<div class="space-y-2">
						<Label for="username">Username</Label>
						<Input
							id="username"
							type="text"
							placeholder="Enter username"
							bind:value={username}
							required
							autocomplete={mode === 'login' ? 'username' : 'username'}
							disabled={loading}
						/>
					</div>

					<div class="space-y-2">
						<Label for="password">Master Password</Label>
						<Input
							id="password"
							type="password"
							placeholder={mode === 'register' ? 'At least 8 characters' : 'Enter master password'}
							bind:value={password}
							required
							autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
							disabled={loading}
						/>
					</div>

					{#if mode === 'register'}
						<div class="space-y-2">
							<Label for="confirm-password">Confirm Password</Label>
							<Input
								id="confirm-password"
								type="password"
								placeholder="Repeat your password"
								bind:value={confirmPassword}
								required
								autocomplete="new-password"
								disabled={loading}
							/>
						</div>
					{/if}
				</CardContent>
				<CardFooter class="flex flex-col gap-3">
					<Button type="submit" class="w-full" disabled={loading}>
						{#if loading}
							<svg class="animate-spin w-4 h-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
						{/if}
						{mode === 'login' ? 'Sign in' : 'Create account'}
					</Button>
					<button
						type="button"
						class="text-sm text-muted-foreground hover:text-foreground transition-colors"
						onclick={toggleMode}
					>
						{mode === 'login'
							? "Don't have an account? Sign up"
							: 'Already have an account? Sign in'}
					</button>
				</CardFooter>
			</form>
		</Card>

		<p class="text-center text-xs text-muted-foreground mt-6">
			End-to-end encrypted · AES-256-GCM · Argon2id
		</p>
	</div>
</main>
