<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import { createCrewPanelState } from './crew-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createCrewPanelState(null, {
    applyCrew: (input) => gateway.applyCrew(input),
  });

  $effect(() => {
    untrack(() => state.sync(snapshot));
  });
</script>

<div data-testid="devtools-crew-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3">
    <h4 class="text-sm font-semibold text-zinc-100">Crew</h4>
    <p class="text-xs text-zinc-500">Adjust total crew while keeping assignments visible.</p>
  </div>

  <div class="grid gap-3">
    <label class="grid gap-1.5 text-xs text-zinc-400">
      <span>Total Crew</span>
      <Input
        data-testid="devtools-crew-total-input"
        type="number"
        min="0"
        max="999"
        step="1"
        bind:value={state.crewTotalDraft}
        disabled={!state.snapshot || state.isApplying}
        class="font-mono text-zinc-100"
      />
    </label>

    <div class="grid grid-cols-2 gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2 text-xs font-mono text-zinc-300">
      <div class="grid gap-1">
        <span class="text-zinc-500">Assigned</span>
        <span>{state.snapshot ? state.snapshot.resources.crew.assigned : '—'}</span>
      </div>
      <div class="grid gap-1">
        <span class="text-zinc-500">Available</span>
        <span>{state.snapshot ? state.snapshot.resources.crew.available : '—'}</span>
      </div>
    </div>
  </div>

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-crew-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-crew-apply"
      size="sm"
      onclick={() => state.apply()}
      disabled={!state.snapshot || state.isApplying}
    >
      {state.isApplying ? 'Applying…' : 'Apply'}
    </Button>
  </div>
</div>
