<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import { createResourcesPanelState } from './resources-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createResourcesPanelState(null, {
    applyResources: (input) => gateway.applyResources(input),
  });

  $effect(() => {
    const s = snapshot;
    untrack(() => state.sync(s));
  });
</script>

<div data-testid="devtools-resources-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3 flex items-start justify-between gap-3">
    <div>
      <h4 class="text-sm font-semibold text-zinc-100">Resources</h4>
      <p class="text-xs text-zinc-500">Stage materials and data values, then apply them.</p>
    </div>

    <div class="text-right text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
      <div>Power</div>
      <div>{state.snapshot ? `${state.snapshot.resources.power.available} avail` : 'offline'}</div>
    </div>
  </div>

  <div class="grid gap-3">
    <label class="grid gap-1.5 text-xs text-zinc-400">
      <span>Materials</span>
      <Input
        data-testid="devtools-materials-input"
        type="number"
        min="0"
        max="99999"
        step="1"
        bind:value={state.materialsDraft}
        disabled={!state.snapshot || state.isApplying}
        class="font-mono text-zinc-100"
      />
    </label>

    <label class="grid gap-1.5 text-xs text-zinc-400">
      <span>Data</span>
      <Input
        data-testid="devtools-data-input"
        type="number"
        min="0"
        max="99999"
        step="1"
        bind:value={state.dataDraft}
        disabled={!state.snapshot || state.isApplying}
        class="font-mono text-zinc-100"
      />
    </label>

    <div class="rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="mb-2 text-[0.625rem] font-semibold uppercase tracking-wide text-zinc-500">
        Power (read only)
      </div>

      <div class="grid grid-cols-3 gap-2 text-xs font-mono text-zinc-300">
        <div class="grid gap-1">
          <span class="text-zinc-500">Generated</span>
          <span>{state.snapshot ? state.snapshot.resources.power.generated : '—'}</span>
        </div>
        <div class="grid gap-1">
          <span class="text-zinc-500">Reserved</span>
          <span>{state.snapshot ? state.snapshot.resources.power.reserved : '—'}</span>
        </div>
        <div class="grid gap-1">
          <span class="text-zinc-500">Available</span>
          <span>{state.snapshot ? state.snapshot.resources.power.available : '—'}</span>
        </div>
      </div>
    </div>
  </div>

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-resources-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-resources-apply"
      size="sm"
      onclick={() => state.apply()}
      disabled={!state.snapshot || state.isApplying}
    >
      {state.isApplying ? 'Applying…' : 'Apply'}
    </Button>
  </div>
</div>
