<script lang="ts">
  import { gameState } from '$lib/game/api/state.svelte';
  import { gameGateway } from '$lib/game/api';
  import type {
    GameSnapshot,
    ServiceId,
    ServiceStatus,
    ServiceDisabledReasonCode,
    ServiceFamily,
  } from '$lib/game/api/types';
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import * as Card from '$lib/components/ui/card';
  import Button from '$lib/components/ui/button/button.svelte';
  import { StatPanel } from '$lib/components/ui/stat-panel';
  import { StatRow } from '$lib/components/ui/stat-row';

  let inflight = $state<Set<string>>(new Set());

  async function runInflightAction(
    id: string,
    fn: () => Promise<{ ok: boolean; snapshot: GameSnapshot }>,
  ) {
    if (inflight.has(id)) return;
    inflight = new Set([...inflight, id]);
    try {
      const result = await fn();
      if (result.ok) gameState.applySnapshot(result.snapshot);
    } catch {
      // Silent catch; store updates via event
    } finally {
      inflight = new Set([...inflight].filter((x) => x !== id));
    }
  }

  const disabledReasonLabels: Record<ServiceDisabledReasonCode, string> = {
    capacity: 'No service slots available',
    crew: 'Not enough crew assigned',
    deficit: 'Power deficit in progress',
    'power-cap': 'Power reserve insufficient',
  };

  const familyLabels: Record<ServiceFamily, string> = {
    production: 'Production',
    support: 'Support',
    command: 'Command',
    conversion: 'Conversion',
  };

  const statusColors: Record<ServiceStatus, string> = {
    active: 'text-emerald-400',
    paused: 'text-amber-400',
    disabled: 'text-muted-foreground',
  };

  async function handleActivation(serviceId: ServiceId, active: boolean) {
    await runInflightAction(serviceId, () =>
      gameGateway.setServiceActivation({ serviceId, active }),
    );
  }

  async function handleReprioritize(serviceId: ServiceId, direction: 'up' | 'down') {
    await runInflightAction(serviceId, () =>
      gameGateway.reprioritizeService({ serviceId, direction }),
    );
  }
</script>

<SnapshotGuard loadingMessage="Loading station services...">
  {#snippet children(snapshot)}
    {@const services = snapshot.routes.services}
    <section data-testid="services-header" class="mb-8">
      <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Station Services</h2>
      <p class="text-muted-foreground">
        Manage active services and crew assignments. Services convert resources based on their
        family type.
      </p>
    </section>

    <StatPanel heading="Service Utilization" data-testid="services-utilization" class="mb-8">
      <StatRow
        kind="ratio"
        label="Active"
        used={services.utilization.active}
        total={services.utilization.capacity}
      />
      <StatRow kind="scalar" label="Available" value={services.utilization.available} />
    </StatPanel>

    {#if services.deficitWarnings.length > 0}
      <section data-testid="deficit-warnings" class="mb-8 flex flex-col gap-2">
        {#each services.deficitWarnings as warning (warning.code)}
          <div class="rounded-lg border border-amber-500 bg-amber-950/30 p-4">
            <h3 class="font-semibold text-amber-200">{warning.title}</h3>
            <p class="mt-1 text-sm text-amber-200/80">{warning.body}</p>
          </div>
        {/each}
      </section>
    {/if}

    <div class="grid gap-6 lg:grid-cols-2">
      {#each services.services as service (service.id)}
        {@const isInflight = inflight.has(service.id)}
        {@const visibleDisabledReasons = service.disabledReasons.filter(
          (reason) => reason !== 'capacity',
        )}
        <Card.Root>
          <Card.Header>
            <div class="flex items-center justify-between">
              <Card.Title>{service.name}</Card.Title>
              <span class="text-xs {statusColors[service.status]}">
                {service.statusLabel}
              </span>
            </div>
            <Card.Description>{service.description}</Card.Description>
          </Card.Header>
          <Card.Content class="flex flex-col gap-4">
            <div class="flex flex-wrap gap-4 text-sm">
              <span class="text-muted-foreground">
                {familyLabels[service.family]}
              </span>
              <span class="text-muted-foreground">
                Priority {service.priorityOrder}
              </span>
            </div>

            <dl class="grid grid-cols-[auto_1fr_auto] items-center gap-x-6 gap-y-2">
              <StatRow
                kind="ratio"
                label="Crew"
                used={service.crewAssignment.current}
                total={service.crewAssignment.required}
              />
              <StatRow
                kind="scalar"
                label="Power Upkeep"
                value={`${service.powerUsage.upkeep}`}
                unit="/s"
              />
              <StatRow
                kind="scalar"
                label="Power Output"
                value={`+${service.powerUsage.output}`}
                unit="/s"
              />
            </dl>

            {#if visibleDisabledReasons.length > 0}
              <div class="flex flex-col gap-1 rounded border border-rose-500/50 bg-rose-950/20 p-3">
                <span class="text-xs font-medium text-rose-300 uppercase">Disabled</span>
                {#each visibleDisabledReasons as reason (reason)}
                  <span class="text-sm text-rose-200/80">
                    {disabledReasonLabels[reason] || reason}
                  </span>
                {/each}
              </div>
            {/if}
          </Card.Content>
          <Card.Footer class="flex flex-wrap gap-2">
            {#if service.status === 'active'}
              <Button
                variant="outline"
                onclick={() => handleActivation(service.id, false)}
                disabled={isInflight}
                class="flex-1"
              >
                {isInflight ? 'Pausing...' : 'Pause'}
              </Button>
            {:else}
              <Button
                onclick={() => handleActivation(service.id, true)}
                disabled={isInflight || service.disabledReasons.length > 0}
                class="flex-1"
              >
                {isInflight ? 'Activating...' : 'Activate'}
              </Button>
            {/if}
            <div class="flex gap-1">
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleReprioritize(service.id, 'up')}
                disabled={isInflight || service.priorityOrder <= 1}
                title="Move up in priority"
              >
                ↑
              </Button>
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleReprioritize(service.id, 'down')}
                disabled={isInflight || service.priorityOrder >= services.services.length}
                title="Move down in priority"
              >
                ↓
              </Button>
            </div>
          </Card.Footer>
        </Card.Root>
      {/each}
    </div>
  {/snippet}
</SnapshotGuard>
