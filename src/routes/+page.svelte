<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";

	let name = $state("");
	let greetMsg = $state("");

	async function greet(event: Event) {
		event.preventDefault();
		greetMsg = await invoke("greet", { name });
	}
</script>

<main class="flex min-h-screen flex-col items-center justify-center gap-8 p-8 text-center">
	<h1 class="text-3xl font-bold tracking-tight">Welcome to Tauri + Svelte</h1>

	<div class="flex items-center gap-4">
		<a href="https://vite.dev" target="_blank">
			<img src="/vite.svg" class="logo vite h-20 p-5 transition-all duration-700 will-change-[filter]" alt="Vite Logo" />
		</a>
		<a href="https://tauri.app" target="_blank">
			<img src="/tauri.svg" class="logo tauri h-20 p-5 transition-all duration-700 will-change-[filter]" alt="Tauri Logo" />
		</a>
		<a href="https://svelte.dev" target="_blank">
			<img src="/svelte.svg" class="logo svelte-kit h-20 p-5 transition-all duration-700 will-change-[filter]" alt="SvelteKit Logo" />
		</a>
	</div>

	<p class="text-muted-foreground">Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

	<form class="flex items-center gap-2" onsubmit={greet}>
		<Input bind:value={name} placeholder="Enter a name..." class="w-48" />
		<Button type="submit">Greet</Button>
	</form>

	<p class="text-sm">{greetMsg}</p>
</main>

<style>
	.logo.vite:hover {
		filter: drop-shadow(0 0 2em #747bff);
	}
	.logo.tauri:hover {
		filter: drop-shadow(0 0 2em #24c8db);
	}
	.logo.svelte-kit:hover {
		filter: drop-shadow(0 0 2em #ff3e00);
	}
</style>