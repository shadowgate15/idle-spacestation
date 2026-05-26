<script lang="ts">
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import { StatPanel } from '$lib/components/ui/stat-panel';
  import { StatRow, type StatRowSeverity } from '$lib/components/ui/stat-row';
  import type { ResourceDeltaSnapshot, WarningSnapshot } from '$lib/game/api/types';

  function rateFor(
    resourceId: ResourceDeltaSnapshot['id'],
    deltas: ResourceDeltaSnapshot[],
  ): number {
    return deltas.find((d) => d.id === resourceId)?.deltaPerSecond ?? 0;
  }

  function severityFor(
    warningCodes: ReadonlySet<string>,
    matchers: readonly string[],
  ): StatRowSeverity | undefined {
    for (const code of matchers) {
      if (warningCodes.has(code)) {
        if (code.includes('deficit') || code.includes('critical')) return 'critical';
        return 'warning';
      }
    }
    return undefined;
  }

  function warningSeverityClass(severity: WarningSnapshot['severity']): string {
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
    {@const deltas = overview.resourceDeltas}
    {@const warnings = overview.deficitWarnings ?? []}
    {@const warningCodes = new Set(warnings.map((w) => w.code))}
    {@const powerSeverity = severityFor(warningCodes, ['power-deficit', 'unstable-net-power'])}
    {@const crewSeverity = severityFor(warningCodes, ['insufficient-crew'])}

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

    <div class="mb-8 grid gap-6 lg:grid-cols-2">
      <StatPanel heading="Resources" data-testid="resource-strip">
        <StatRow
          kind="stock"
          label="Materials"
          value={Math.floor(snapshot.resources.materials)}
          perSecond={rateFor('materials', deltas)}
        />
        <StatRow
          kind="stock"
          label="Data"
          value={Math.floor(snapshot.resources.data)}
          perSecond={rateFor('data', deltas)}
        />
      </StatPanel>

      <StatPanel heading="Crew & Power" data-testid="crew-power-panel">
        <StatRow
          kind="ratio"
          label="Crew"
          used={snapshot.resources.crew.assigned}
          total={snapshot.resources.crew.total}
          severity={crewSeverity}
        />
        <StatRow
          kind="capacity"
          label="Power"
          current={snapshot.resources.power.available}
          max={snapshot.resources.power.generated}
          perSecond={rateFor('power', deltas)}
          precision={1}
          severity={powerSeverity}
        />
      </StatPanel>
    </div>

    {#if warnings.length > 0}
      <section data-testid="deficit-warnings" class="mb-8 flex flex-col gap-2">
        {#each warnings as warning (warning.code)}
          <div class="rounded-lg border p-4 {warningSeverityClass(warning.severity)}">
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
        <StatPanel
          heading={`Station Tier — ${overview.stationTier.label} of ${overview.stationTier.max}`}
          data-testid="overview-panel"
          id="overview-panel"
        >
          <StatRow
            kind="progress"
            label="Tier"
            current={overview.stationTier.current}
            goal={overview.stationTier.max}
          />
          <StatRow kind="label" label="Active Slots" value={overview.serviceUtilization.summary} />
          <StatRow kind="label" label="Survey" value={overview.surveyProgress.summary} />
        </StatPanel>

        <StatPanel heading="Service Utilization" data-testid="station-stats" id="station-stats">
          <StatRow
            kind="ratio"
            label="Active"
            used={overview.serviceUtilization.active}
            total={overview.serviceUtilization.capacity}
          />
          <StatRow kind="scalar" label="Available" value={overview.serviceUtilization.available} />
        </StatPanel>
      </div>

      <StatPanel heading="Survey Progress" data-testid="survey-panel" id="survey-panel">
        <StatRow kind="scalar" label="Current" value={overview.surveyProgress.current} />
        {#if overview.surveyProgress.nextPlanetName}
          <StatRow
            kind="label"
            label="Next Target"
            value={overview.surveyProgress.nextPlanetName}
          />
          {#if overview.surveyProgress.nextThreshold}
            <StatRow
              kind="progress"
              label="Threshold"
              current={overview.surveyProgress.current}
              goal={overview.surveyProgress.nextThreshold}
            />
          {/if}
        {/if}
      </StatPanel>
    </div>
  {/snippet}
</SnapshotGuard>
