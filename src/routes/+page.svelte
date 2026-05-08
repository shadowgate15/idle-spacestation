<script lang="ts">
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import { StatTile, type StatTileVariant } from '$lib/components/ui/stat-tile';
  import type { ResourceDeltaSnapshot, WarningSnapshot } from '$lib/game/api/types';

  function formatDelta(delta: ResourceDeltaSnapshot): string {
    const sign = delta.deltaPerSecond >= 0 ? '+' : '';
    return `${sign}${delta.deltaPerSecond.toFixed(1)}/s`;
  }

  function trendVariant(trend: ResourceDeltaSnapshot['trend']): StatTileVariant | undefined {
    if (trend === 'positive') return 'positive';
    if (trend === 'negative') return 'negative';
    return undefined;
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
          <StatTile
            label={resource.label}
            value={formatDelta(resource)}
            variant={trendVariant(resource.trend)}
          />
        {/each}
      </dl>
    </section>

    <section class="mb-8 rounded-lg border border-border bg-card p-4" data-testid="stockpile-strip">
      <dl class="flex flex-wrap gap-6">
        <StatTile label="Materials" value={Math.floor(snapshot.resources.materials)} />
        <StatTile label="Data" value={Math.floor(snapshot.resources.data)} />
        <StatTile
          label="Crew"
          value="{snapshot.resources.crew.assigned} / {snapshot.resources.crew.total}"
        />
        <StatTile
          label="Power"
          value="{snapshot.resources.power.available.toFixed(
            1,
          )} available of {snapshot.resources.power.generated.toFixed(1)} generated"
        />
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
            <StatTile label="Active" value={overview.serviceUtilization.active} />
            <StatTile label="Capacity" value={overview.serviceUtilization.capacity} />
            <StatTile label="Available" value={overview.serviceUtilization.available} />
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
          <StatTile label="Current" value={overview.surveyProgress.current} />
          {#if overview.surveyProgress.nextPlanetName}
            <StatTile label="Next Target" value={overview.surveyProgress.nextPlanetName} />
            {#if overview.surveyProgress.nextThreshold}
              <StatTile label="Threshold" value={overview.surveyProgress.nextThreshold} />
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
