<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { gameGateway } from '$lib/game/api';
  import type { ServiceId, ServicesViewModel, ServiceStatus } from '$lib/game/api/types';
  import * as Card from '$lib/components/ui/card';
  import Button from '$lib/components/ui/button/button.svelte';

  type AppRoute = '/' | '/systems' | '/services' | '/planets' | '/prestige';

  let services = $state<ServicesViewModel | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let acting = $state<Set<string>>(new Set());
  let isPolling = $state(false);
  let destroyed = $state(false);
  let pollInterval = $state<ReturnType<typeof setInterval> | null>(null);

  const disabledReasonLabels: Record<string, string> = {
    capacity: 'No service slots available',
    crew: 'Not enough crew assigned',
    deficit: 'Power deficit in progress',
    'power-cap': 'Power reserve insufficient',
  };

  const familyLabels: Record<string, string> = {
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

  async function loadServices() {
    if (isPolling) return;

    isPolling = true;
    try {
      const snapshot = await gameGateway.getSnapshot();
      if (destroyed) return;
      services = snapshot.routes.services;
      error = null;
    } catch (e) {
      if (destroyed) return;
      error = e instanceof Error ? e.message : 'Failed to load services data';
    } finally {
      isPolling = false;
    }
  }

  onMount(() => {
    destroyed = false;

    const initialize = async () => {
      try {
        await loadServices();
      } finally {
        if (destroyed) return;
        loading = false;
      }

      if (destroyed) return;
      pollInterval = setInterval(async () => {
        await loadServices();
      }, 1000);
    };

    void initialize();

    return () => {
      destroyed = true;
      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
    };
  });

  async function handleActivation(serviceId: ServiceId, active: boolean) {
    if (acting.has(serviceId)) return;

    acting = new Set([...acting, serviceId]);
    try {
      const result = await gameGateway.setServiceActivation({
        serviceId,
        active,
      });
      if (result.ok) {
        services = result.snapshot.routes.services;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : `Failed to ${active ? 'activate' : 'pause'} service`;
    } finally {
      acting = new Set([...acting].filter((id) => id !== serviceId));
    }
  }

  async function handleReprioritize(serviceId: ServiceId, direction: 'up' | 'down') {
    if (acting.has(serviceId)) return;

    acting = new Set([...acting, serviceId]);
    try {
      const result = await gameGateway.reprioritizeService({
        serviceId,
        direction,
      });
      if (result.ok) {
        services = result.snapshot.routes.services;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to reprioritize service';
    } finally {
      acting = new Set([...acting].filter((id) => id !== serviceId));
    }
  }

  function navigateTo(path: AppRoute) {
    goto(resolve(path));
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-12">
    <p class="text-muted-foreground">Loading station services...</p>
  </div>
{:else if error}
  <div class="rounded-lg border border-rose-500 bg-rose-950/30 p-4">
    <p class="text-rose-200">{error}</p>
  </div>
{:else if services}
  <section data-testid="services-header" class="mb-8">
    <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Station Services</h2>
    <p class="text-muted-foreground">
      Manage active services and crew assignments. Services convert resources based on their family
      type.
    </p>
  </section>

  <section class="mb-8 rounded-lg border border-border bg-card p-4">
    <div class="flex flex-wrap gap-6">
      <div class="flex flex-col">
        <span class="text-xs tracking-wide text-muted-foreground uppercase">Active</span>
        <span class="text-lg font-bold text-foreground">{services.utilization.active}</span>
      </div>
      <div class="flex flex-col">
        <span class="text-xs tracking-wide text-muted-foreground uppercase">Capacity</span>
        <span class="text-lg font-bold text-foreground">{services.utilization.capacity}</span>
      </div>
      <div class="flex flex-col">
        <span class="text-xs tracking-wide text-muted-foreground uppercase">Available</span>
        <span class="text-lg font-bold text-foreground">{services.utilization.available}</span>
      </div>
    </div>
  </section>

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

  <nav class="mb-8 flex gap-4">
    <button
      type="button"
      class="text-sm text-muted-foreground transition-colors hover:text-foreground"
      onclick={() => navigateTo('/')}
    >
      Overview
    </button>
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
    {#each services.services as service (service.id)}
      {@const isActing = acting.has(service.id)}
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

          <div class="flex flex-wrap gap-4">
            <div class="flex flex-col">
              <span class="text-xs tracking-wide text-muted-foreground uppercase"> Crew </span>
              <span class="text-sm font-medium text-foreground">
                {service.crewAssignment.current} / {service.crewAssignment.required}
              </span>
            </div>
            <div class="flex flex-col">
              <span class="text-xs tracking-wide text-muted-foreground uppercase">
                Power Upkeep
              </span>
              <span class="text-sm font-medium text-foreground">
                {service.powerUsage.upkeep} /s
              </span>
            </div>
            <div class="flex flex-col">
              <span class="text-xs tracking-wide text-muted-foreground uppercase">
                Power Output
              </span>
              <span class="text-sm font-medium text-foreground">
                +{service.powerUsage.output} /s
              </span>
            </div>
          </div>

          {#if service.disabledReasons.length > 0}
            <div class="flex flex-col gap-1 rounded border border-rose-500/50 bg-rose-950/20 p-3">
              <span class="text-xs font-medium text-rose-300 uppercase">Disabled</span>
              {#each service.disabledReasons as reason (reason)}
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
              disabled={isActing}
              class="flex-1"
            >
              {isActing ? 'Pausing...' : 'Pause'}
            </Button>
          {:else}
            <Button
              onclick={() => handleActivation(service.id, true)}
              disabled={isActing || service.disabledReasons.length > 0}
              class="flex-1"
            >
              {isActing ? 'Activating...' : 'Activate'}
            </Button>
          {/if}
          <div class="flex gap-1">
            <Button
              variant="outline"
              size="sm"
              onclick={() => handleReprioritize(service.id, 'up')}
              disabled={isActing || service.priorityOrder <= 1}
              title="Move up in priority"
            >
              ↑
            </Button>
            <Button
              variant="outline"
              size="sm"
              onclick={() => handleReprioritize(service.id, 'down')}
              disabled={isActing || service.priorityOrder >= services.services.length}
              title="Move down in priority"
            >
              ↓
            </Button>
          </div>
        </Card.Footer>
      </Card.Root>
    {/each}
  </div>
{/if}
