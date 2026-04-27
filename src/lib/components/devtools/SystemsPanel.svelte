<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot, SystemId } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import { createSystemsPanelState } from './systems-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createSystemsPanelState(null, {
    applySystems: (input) => gateway.applySystems(input),
  });

  $effect(() => {
    untrack(() => state.sync(snapshot));
  });

  function getCurrentLevel(id: SystemId) {
    return state.snapshot?.systems.find((system) => system.id === id)?.level ?? null;
  }
</script>

<div data-testid="devtools-systems-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3">
    <h4 class="text-sm font-semibold text-zinc-100">Systems</h4>
    <p class="text-xs text-zinc-500">Stage system levels for the whole station, then apply them together.</p>
  </div>

  {#if state.drafts.length > 0}
    <div class="grid gap-3">
      {#each state.drafts as systemDraft (systemDraft.id)}
        {@const currentLevel = getCurrentLevel(systemDraft.id)}

        <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-xs font-semibold text-zinc-200">{systemDraft.id}</div>
              <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
                Current level: {currentLevel ?? '—'}
              </div>
            </div>
          </div>

          <label class="grid gap-1.5 text-xs text-zinc-400">
            <span>Level</span>
            <Input
              data-testid={`devtools-system-${systemDraft.id}-level`}
              type="number"
              min="1"
              max="4"
              step="1"
              bind:value={systemDraft.level}
              disabled={state.isApplying}
              class="font-mono text-zinc-100"
            />
          </label>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-xs text-zinc-500">Snapshot unavailable.</p>
  {/if}

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-systems-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-systems-apply"
      size="sm"
      onclick={() => state.apply()}
      disabled={!state.snapshot || state.isApplying}
    >
      {state.isApplying ? 'Applying…' : 'Apply All'}
    </Button>
  </div>
</div>
