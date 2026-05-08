<script lang="ts">
  import type { Snippet } from 'svelte';
  import { gameState } from '$lib/game/api/state.svelte';
  import type { GameSnapshot } from '$lib/game/api/types';

  let {
    loadingMessage = 'Loading...',
    children,
  }: {
    loadingMessage?: string;
    children: Snippet<[GameSnapshot]>;
  } = $props();

  const status = $derived(gameState.status);
  const errorMessage = $derived(gameState.error?.message ?? null);
  const snapshot = $derived(gameState.snapshot);
</script>

{#if status !== 'ready'}
  <div class="flex items-center justify-center py-12">
    <p class="text-muted-foreground">{loadingMessage}</p>
  </div>
{:else if errorMessage}
  <div class="rounded-lg border border-rose-500 bg-rose-950/30 p-4">
    <p class="text-rose-200">{errorMessage}</p>
  </div>
{:else if snapshot}
  {@render children(snapshot)}
{/if}
