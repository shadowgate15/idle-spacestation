<script lang="ts">
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import { StatPanel } from '$lib/components/ui/stat-panel';
  import { StatRow } from '$lib/components/ui/stat-row';

  function getPlanetStatusClass(discovered: boolean, active: boolean): string {
    if (active) return 'border-emerald-500 bg-emerald-950/30';
    if (discovered) return 'border-border bg-card';
    return 'border-dashed border-border bg-card/50 opacity-60';
  }
</script>

<SnapshotGuard loadingMessage="Loading planets data...">
  {#snippet children(snapshot)}
    {@const planets = snapshot.routes.planets}
    <section data-testid="planets-hero" class="mb-8">
      <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Planetary Operations</h2>
      <p class="mb-4 text-muted-foreground">
        Track discovery progress and review which planets will be available for future runs.
      </p>
    </section>

    <div data-testid="survey-progress" class="mb-8">
      <StatPanel heading="Survey Progress">
        <StatRow kind="scalar" label="Current" value={planets.surveyProgress.current} />
        {#if planets.surveyProgress.nextPlanetName}
          <StatRow kind="label" label="Next Target" value={planets.surveyProgress.nextPlanetName} />
          {#if planets.surveyProgress.nextThreshold}
            <StatRow
              kind="progress"
              label="Threshold"
              current={planets.surveyProgress.current}
              goal={planets.surveyProgress.nextThreshold}
            />
          {/if}
        {/if}
      </StatPanel>
      {#if planets.surveyProgress.summary}
        <p class="mt-2 text-sm text-muted-foreground">
          {planets.surveyProgress.summary}
        </p>
      {/if}
    </div>

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
  {/snippet}
</SnapshotGuard>
