<script lang="ts">
  import { onMount } from 'svelte';
  import { gameGateway } from '$lib/game/api';
  import type { PlanetsViewModel } from '$lib/game/api/types';

  let planets = $state<PlanetsViewModel | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const snapshot = await gameGateway.getSnapshot();
      planets = snapshot.routes.planets;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load planets data';
    } finally {
      loading = false;
    }
  });

  function getPlanetStatusClass(discovered: boolean, active: boolean): string {
    if (active) return 'border-emerald-500 bg-emerald-950/30';
    if (discovered) return 'border-border bg-card';
    return 'border-dashed border-border bg-card/50 opacity-60';
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-12">
    <p class="text-muted-foreground">Loading planets data...</p>
  </div>
{:else if error}
  <div class="rounded-lg border border-rose-500 bg-rose-950/30 p-4">
    <p class="text-rose-200">{error}</p>
  </div>
{:else if planets}
  <section data-testid="planets-hero" class="mb-8">
    <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Planetary Operations</h2>
    <p class="mb-4 text-muted-foreground">
      Track discovery progress and review which planets will be available for future runs.
    </p>
  </section>

  <section data-testid="survey-progress" class="mb-8 rounded-lg border border-border bg-card p-4">
    <h3 class="mb-3 text-base font-semibold text-foreground">Survey Progress</h3>
    <dl class="flex flex-wrap gap-6">
      <div class="flex flex-col">
        <dt class="text-xs tracking-wide text-muted-foreground uppercase">Current</dt>
        <dd class="text-lg font-bold text-foreground">
          {planets.surveyProgress.current}
        </dd>
      </div>
      {#if planets.surveyProgress.nextPlanetName}
        <div class="flex flex-col">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Next Target</dt>
          <dd class="text-lg font-bold text-foreground">
            {planets.surveyProgress.nextPlanetName}
          </dd>
        </div>
        {#if planets.surveyProgress.nextThreshold}
          <div class="flex flex-col">
            <dt class="text-xs tracking-wide text-muted-foreground uppercase">Threshold</dt>
            <dd class="text-lg font-bold text-foreground">
              {planets.surveyProgress.nextThreshold}
            </dd>
          </div>
        {/if}
      {/if}
    </dl>
    {#if planets.surveyProgress.summary}
      <p class="mt-2 text-sm text-muted-foreground">
        {planets.surveyProgress.summary}
      </p>
    {/if}
  </section>

  <section data-testid="planets-list" class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    {#each planets.planets as planet (planet.id)}
      <article
        data-testid="planet-{planet.id}"
        class="rounded-lg border p-4 {getPlanetStatusClass(planet.discovered, planet.active)}"
      >
        <div class="mb-2 flex items-start justify-between">
          <h3 class="font-semibold text-foreground">{planet.name}</h3>
          {#if planet.active}
            <span
              class="inline-flex items-center rounded bg-emerald-950/50 px-2 py-0.5 text-xs text-emerald-200"
            >
              Active
            </span>
          {:else if planet.selectableForNextRun}
            <span
              class="inline-flex items-center rounded bg-sky-950/50 px-2 py-0.5 text-xs text-sky-200"
            >
              Selectable
            </span>
          {/if}
        </div>

        <p class="mb-3 text-sm text-muted-foreground">
          {planet.description}
        </p>

        {#if planet.discovered && planet.modifiers.length > 0}
          <div class="mb-3 flex flex-wrap gap-1.5">
            {#each planet.modifiers as modifier (modifier.target)}
              <span
                class="inline-flex items-center rounded border border-border bg-card px-1.5 py-0.5 text-xs {modifier.percent >=
                0
                  ? 'text-emerald-400'
                  : 'text-rose-400'}"
              >
                {modifier.effectText}
              </span>
            {/each}
          </div>
        {/if}

        <div class="text-xs text-muted-foreground">
          {#if !planet.discovered && planet.surveyThreshold !== null}
            <p>
              Unlocks at {planet.surveyThreshold} survey progress
            </p>
            <p class="mt-1">
              Current: {planet.surveyProgress}
            </p>
          {:else if planet.selectabilityReason}
            <p class="text-amber-400">{planet.selectabilityReason}</p>
          {/if}
        </div>
      </article>
    {/each}
  </section>
{/if}
