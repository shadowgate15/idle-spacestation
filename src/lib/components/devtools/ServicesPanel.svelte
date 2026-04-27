<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot, RawServiceStateSnapshot, ServiceId } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import { createServicesPanelState } from './services-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createServicesPanelState(null, {
    applyServices: (input) => gateway.applyServices(input),
  });

  $effect(() => {
    untrack(() => state.sync(snapshot));
  });

  function getCurrentService(id: ServiceId): RawServiceStateSnapshot | null {
    return state.snapshot?.services.find((service) => service.id === id) ?? null;
  }

  function formatPauseReason(reason: RawServiceStateSnapshot['pauseReason']) {
    return reason ?? 'none';
  }
</script>

<div data-testid="devtools-services-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3">
    <h4 class="text-sm font-semibold text-zinc-100">Services</h4>
    <p class="text-xs text-zinc-500">Stage service activation, staffing, and ordering, then apply the full set.</p>
  </div>

  {#if state.drafts.length > 0}
    <div class="grid gap-3">
      {#each state.drafts as serviceDraft (serviceDraft.id)}
        {@const currentService = getCurrentService(serviceDraft.id)}

        <div class="grid gap-3 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-xs font-semibold text-zinc-200">{serviceDraft.id}</div>
              <div class="mt-1 grid gap-0.5 text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
                <div>Active: {currentService ? String(currentService.isActive) : '—'}</div>
                <div>Paused: {currentService ? String(currentService.isPaused) : '—'}</div>
                <div>Pause reason: {currentService ? formatPauseReason(currentService.pauseReason) : '—'}</div>
              </div>
            </div>
          </div>

          <label class="flex items-center gap-2 text-xs text-zinc-400">
            <input
              data-testid={`devtools-service-${serviceDraft.id}-active`}
              type="checkbox"
              bind:checked={serviceDraft.desiredActive}
              disabled={state.isApplying}
              class="size-4 rounded border-zinc-700 bg-zinc-950 text-zinc-100 accent-zinc-100"
            />
            <span>Desired active</span>
          </label>

          <div class="grid grid-cols-2 gap-3">
            <label class="grid gap-1.5 text-xs text-zinc-400">
              <span>Assigned crew</span>
              <Input
                data-testid={`devtools-service-${serviceDraft.id}-crew`}
                type="number"
                min="0"
                step="1"
                bind:value={serviceDraft.assignedCrew}
                disabled={state.isApplying}
                class="font-mono text-zinc-100"
              />
            </label>

            <label class="grid gap-1.5 text-xs text-zinc-400">
              <span>Priority</span>
              <Input
                data-testid={`devtools-service-${serviceDraft.id}-priority`}
                type="number"
                min="1"
                max={state.drafts.length}
                step="1"
                bind:value={serviceDraft.priority}
                disabled={state.isApplying}
                class="font-mono text-zinc-100"
              />
            </label>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-xs text-zinc-500">Snapshot unavailable.</p>
  {/if}

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-services-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-services-apply"
      size="sm"
      onclick={() => state.apply()}
      disabled={!state.snapshot || state.isApplying}
    >
      {state.isApplying ? 'Applying…' : 'Apply All'}
    </Button>
  </div>
</div>
