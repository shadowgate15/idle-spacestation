<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { DoctrineId, GameSnapshot, PlanetId } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import {
    createProgressionPanelState,
    doctrineIds,
    planetIds,
  } from './progression-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createProgressionPanelState(null, {
    applyProgression: (input) => gateway.applyProgression(input),
  });

  $effect(() => {
    untrack(() => state.sync(snapshot));
  });

  function toggleDoctrine(id: DoctrineId, checked: boolean) {
    state.toggleUnlockedDoctrine(id, checked);
  }

  function togglePlanet(id: PlanetId, checked: boolean) {
    state.toggleDiscoveredPlanet(id, checked);
  }
</script>

<div data-testid="devtools-progression-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3">
    <h4 class="text-sm font-semibold text-zinc-100">Progression</h4>
    <p class="text-xs text-zinc-500">Stage doctrine and exploration progression changes, then apply them together.</p>
  </div>

  <div class="grid gap-3">
    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
        Current fragments: {state.snapshot?.run.doctrineFragments ?? '—'}
      </div>

      <label class="grid gap-1.5 text-xs text-zinc-400">
        <span>Doctrine fragments</span>
        <Input
          data-testid="devtools-doctrine-fragments-input"
          type="number"
          min="0"
          step="1"
          value={state.draft.doctrineFragments}
          onchange={(event) => state.setDoctrineFragments(event.currentTarget.valueAsNumber)}
          disabled={state.isApplying}
          class="font-mono text-zinc-100"
        />
      </label>
    </div>

    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div>
        <div class="text-xs font-semibold text-zinc-200">Unlocked doctrines</div>
        <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
          Current: {state.snapshot?.run.doctrineIds.join(', ') || 'none'}
        </div>
      </div>

      <div class="grid gap-2">
        {#each doctrineIds as doctrineId (doctrineId)}
          <label class="flex items-center gap-2 text-xs text-zinc-400">
            <input
              data-testid={`devtools-doctrine-${doctrineId}-checkbox`}
              type="checkbox"
              checked={state.draft.unlockedDoctrines.includes(doctrineId)}
              disabled={state.isApplying}
              onclick={(event) => toggleDoctrine(doctrineId, event.currentTarget.checked)}
              class="size-4 rounded border-zinc-700 bg-zinc-950 text-zinc-100 accent-zinc-100"
            />
            <span>{doctrineId}</span>
          </label>
        {/each}
      </div>
    </div>

    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div>
        <div class="text-xs font-semibold text-zinc-200">Discovered planets</div>
        <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
          Current: {state.snapshot?.run.discoveredPlanetIds.join(', ') || 'none'}
        </div>
      </div>

      <div class="grid gap-2">
        {#each planetIds as planetId (planetId)}
          <label class="flex items-center gap-2 text-xs text-zinc-400">
            <input
              data-testid={`devtools-planet-${planetId}-discovered`}
              type="checkbox"
              checked={state.draft.discoveredPlanets.includes(planetId)}
              disabled={state.isApplying || planetId === 'solstice-anchor'}
              onclick={(event) => togglePlanet(planetId, event.currentTarget.checked)}
              class="size-4 rounded border-zinc-700 bg-zinc-950 text-zinc-100 accent-zinc-100"
            />
            <span>{planetId}</span>
          </label>
        {/each}
      </div>
    </div>

    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
        Current active planet: {state.snapshot?.run.activePlanetId ?? '—'}
      </div>

      <label class="grid gap-1.5 text-xs text-zinc-400">
        <span>Active planet</span>
        <select
          data-testid="devtools-active-planet-select"
          value={state.draft.activePlanet}
          onchange={(event) => state.setActivePlanet(event.currentTarget.value as PlanetId)}
          disabled={state.isApplying}
          class={cn(
            'flex h-7 w-full min-w-0 rounded-md border border-input bg-input/20 px-2 py-0.5 font-mono text-sm text-zinc-100 outline-none transition-colors focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-xs/relaxed dark:bg-input/30',
          )}
        >
          {#each state.activePlanetOptions as planetId (planetId)}
            <option value={planetId}>{planetId}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
        Current survey progress: {state.snapshot?.run.surveyProgress ?? '—'}
      </div>

      <label class="grid gap-1.5 text-xs text-zinc-400">
        <span>Survey progress</span>
        <Input
          data-testid="devtools-survey-progress-input"
          type="number"
          min="0"
          max="1"
          step="0.01"
          value={state.draft.surveyProgress}
          onchange={(event) => state.setSurveyProgress(event.currentTarget.valueAsNumber)}
          disabled={state.isApplying}
          class="font-mono text-zinc-100"
        />
      </label>
    </div>
  </div>

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-progression-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-progression-apply"
      size="sm"
      onclick={() => state.apply()}
      disabled={!state.snapshot || state.isApplying}
    >
      {state.isApplying ? 'Applying…' : 'Apply'}
    </Button>
  </div>
</div>
