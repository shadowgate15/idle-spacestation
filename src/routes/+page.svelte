<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import type { ResourceDeltaSnapshot, WarningSnapshot } from '$lib/game/api/types';

  type AppRoute = '/' | '/systems' | '/services' | '/planets' | '/prestige';

  function formatDelta(delta: ResourceDeltaSnapshot): string {
    const sign = delta.deltaPerSecond >= 0 ? '+' : '';
    return `${sign}${delta.deltaPerSecond.toFixed(1)}/s`;
  }

  function getTrendClass(trend: ResourceDeltaSnapshot['trend']): string {
    switch (trend) {
      case 'positive':
        return 'text-emerald-400';
      case 'negative':
        return 'text-rose-400';
      default:
        return 'text-muted-foreground';
    }
  }

  function getWarningSeverityClass(severity: WarningSnapshot['severity']): string {
    switch (severity) {
      case 'critical':
        return 'border-rose-500 bg-rose-950/30 text-rose-200';
      case 'warning':
        return 'border-amber-500 bg-amber-950/30 text-amber-200';
      default:
        return 'border-sky-500 bg-sky-950/30 text-sky-200';
    }
  }

  function navigateTo(path: AppRoute) {
    goto(resolve(path));
  }
</script>

<SnapshotGuard loadingMessage="Loading station data...">
  {#snippet children(snapshot)}
    {@const overview = snapshot.routes.overview}
    {@const deficitWarnings = overview.deficitWarnings ?? []}
    <section data-testid="home-hero" class="mb-8">
      <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">
        Station Command — {overview.activePlanet.name}
      </h2>
      <p class="mb-4 text-muted-foreground">
        {overview.activePlanet.description}
      </p>
      {#if overview.activePlanet.modifiers.length > 0}
        <div class="mb-4 flex flex-wrap gap-2">
          {#each overview.activePlanet.modifiers as modifier (modifier.target)}
            <span
              class="inline-flex items-center rounded border border-border bg-card px-2 py-0.5 text-xs text-muted-foreground"
            >
              {modifier.effectText}
            </span>
          {/each}
        </div>
      {/if}
    </section>

    <section
      data-testid="resource-strip"
      class="mb-8 flex gap-6 rounded-lg border border-border bg-card p-4"
    >
      <dl class="flex flex-wrap gap-6">
        {#each overview.resourceDeltas as resource (resource.id)}
          <div class="flex flex-col">
            <dt class="text-xs tracking-wide text-muted-foreground uppercase">
              {resource.label}
            </dt>
            <dd class="text-lg font-bold {getTrendClass(resource.trend)}">
              {formatDelta(resource)}
            </dd>
          </div>
        {/each}
      </dl>
    </section>

    <section class="mb-8 rounded-lg border border-border bg-card p-4" data-testid="stockpile-strip">
      <dl class="flex flex-wrap gap-6">
        <div class="flex flex-col">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Materials</dt>
          <dd class="text-lg font-bold text-foreground">
            {Math.floor(snapshot.resources.materials)}
          </dd>
        </div>
        <div class="flex flex-col">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Data</dt>
          <dd class="text-lg font-bold text-foreground">
            {Math.floor(snapshot.resources.data)}
          </dd>
        </div>
        <div class="flex flex-col">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Crew</dt>
          <dd class="text-lg font-bold text-foreground">
            {snapshot.resources.crew.assigned} / {snapshot.resources.crew.total}
          </dd>
        </div>
        <div class="flex flex-col">
          <dt class="text-xs tracking-wide text-muted-foreground uppercase">Power</dt>
          <dd class="text-lg font-bold text-foreground">
            {snapshot.resources.power.available.toFixed(1)} available of
            {snapshot.resources.power.generated.toFixed(1)} generated
          </dd>
        </div>
      </dl>
    </section>

    {#if deficitWarnings.length > 0}
      <section data-testid="deficit-warnings" class="mb-8 flex flex-col gap-2">
        {#each deficitWarnings as warning (warning.code)}
          <div class="rounded-lg border p-4 {getWarningSeverityClass(warning.severity)}">
            <div class="flex items-center gap-2">
              {#if warning.severity === 'critical'}
                <span class="text-lg">⚠</span>
              {:else if warning.severity === 'warning'}
                <span class="text-lg">⚡</span>
              {:else}
                <span class="text-lg">ℹ</span>
              {/if}
              <h3 class="font-semibold">{warning.title}</h3>
            </div>
            <p class="mt-1 text-sm">{warning.body}</p>
          </div>
        {/each}
      </section>
    {/if}

    <nav class="mb-8 flex gap-4">
      <button
        type="button"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        onclick={() => navigateTo('/systems')}
      >
        Systems
      </button>
      <button
        type="button"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        onclick={() => navigateTo('/services')}
      >
        Services
      </button>
      <button
        type="button"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        onclick={() => navigateTo('/planets')}
      >
        Planets
      </button>
      <button
        type="button"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        onclick={() => navigateTo('/prestige')}
      >
        Prestige
      </button>
    </nav>

    <div class="grid gap-6 lg:grid-cols-2">
      <div class="flex flex-col gap-6">
        <section
          id="overview-panel"
          data-testid="overview-panel"
          class="rounded-lg border border-border bg-card p-4"
        >
          <h2 class="mb-3 text-base font-semibold text-foreground">
            Station Tier — {overview.stationTier.label} of {overview.stationTier.max}
          </h2>
          <ul class="space-y-2 text-sm text-muted-foreground">
            <li>Active service slots: {overview.serviceUtilization.summary}</li>
            <li>Survey progress: {overview.surveyProgress.summary}</li>
          </ul>
        </section>

        <section
          id="station-stats"
          data-testid="station-stats"
          class="rounded-lg border border-border bg-card p-4"
        >
          <h2 class="mb-3 text-base font-semibold text-foreground">Service Utilization</h2>
          <dl class="flex gap-6">
            <div class="flex flex-col">
              <dt class="text-xs tracking-wide text-muted-foreground uppercase">Active</dt>
              <dd class="text-lg font-bold text-foreground">
                {overview.serviceUtilization.active}
              </dd>
            </div>
            <div class="flex flex-col">
              <dt class="text-xs tracking-wide text-muted-foreground uppercase">Capacity</dt>
              <dd class="text-lg font-bold text-foreground">
                {overview.serviceUtilization.capacity}
              </dd>
            </div>
            <div class="flex flex-col">
              <dt class="text-xs tracking-wide text-muted-foreground uppercase">Available</dt>
              <dd class="text-lg font-bold text-foreground">
                {overview.serviceUtilization.available}
              </dd>
            </div>
          </dl>
        </section>
      </div>

      <section
        id="survey-panel"
        data-testid="survey-panel"
        class="rounded-lg border border-border bg-card p-4"
      >
        <h2 class="mb-3 text-base font-semibold text-foreground">Survey Progress</h2>
        <dl class="flex gap-6">
          <div class="flex flex-col">
            <dt class="text-xs tracking-wide text-muted-foreground uppercase">Current</dt>
            <dd class="text-lg font-bold text-foreground">
              {overview.surveyProgress.current}
            </dd>
          </div>
          {#if overview.surveyProgress.nextPlanetName}
            <div class="flex flex-col">
              <dt class="text-xs tracking-wide text-muted-foreground uppercase">Next Target</dt>
              <dd class="text-lg font-bold text-foreground">
                {overview.surveyProgress.nextPlanetName}
              </dd>
            </div>
            {#if overview.surveyProgress.nextThreshold}
              <div class="flex flex-col">
                <dt class="text-xs tracking-wide text-muted-foreground uppercase">Threshold</dt>
                <dd class="text-lg font-bold text-foreground">
                  {overview.surveyProgress.nextThreshold}
                </dd>
              </div>
            {/if}
          {/if}
        </dl>
        {#if overview.surveyProgress.summary}
          <p class="mt-2 text-sm text-muted-foreground">
            {overview.surveyProgress.summary}
          </p>
        {/if}
      </section>
    </div>
  {/snippet}
</SnapshotGuard>
