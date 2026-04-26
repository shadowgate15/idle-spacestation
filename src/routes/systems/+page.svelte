<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { gameGateway } from '$lib/game/api';
  import type { SystemId, SystemsViewModel } from '$lib/game/api/types';
  import * as Card from '$lib/components/ui/card';
  import Button from '$lib/components/ui/button/button.svelte';

  type AppRoute = '/' | '/systems' | '/services' | '/planets' | '/prestige';

  let systems = $state<SystemsViewModel | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let upgrading = $state<Set<string>>(new Set());

  onMount(async () => {
    try {
      const snapshot = await gameGateway.getSnapshot();
      systems = snapshot.routes.systems;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load systems data';
    } finally {
      loading = false;
    }
  });

  async function handleUpgrade(systemId: SystemId) {
    if (upgrading.has(systemId)) return;

    upgrading = new Set([...upgrading, systemId]);
    try {
      const result = await gameGateway.upgradeSystem({ systemId });
      if (result.ok) {
        systems = result.snapshot.routes.systems;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to upgrade system';
    } finally {
      upgrading = new Set([...upgrading].filter((id) => id !== systemId));
    }
  }

  function navigateTo(path: AppRoute) {
    goto(resolve(path));
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-12">
    <p class="text-muted-foreground">Loading station systems...</p>
  </div>
{:else if error}
  <div class="rounded-lg border border-rose-500 bg-rose-950/30 p-4">
    <p class="text-rose-200">{error}</p>
  </div>
{:else if systems}
  <section data-testid="systems-header" class="mb-8">
    <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Station Systems</h2>
    <p class="text-muted-foreground">
      Upgrade to expand station capabilities. Each system gates specific limits.
    </p>
  </section>

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
    {#each systems.systems as system (system.id)}
      {@const isUpgrading = upgrading.has(system.id)}
      <Card.Root>
        <Card.Header>
          <Card.Title>{system.name}</Card.Title>
          <Card.Description>{system.description}</Card.Description>
        </Card.Header>
        <Card.Content class="flex flex-col gap-4">
          <div class="flex items-center gap-4">
            <span class="text-sm text-muted-foreground">Level</span>
            <span class="text-lg font-bold text-foreground">
              {system.level} / {system.maxLevel}
            </span>
          </div>

          <dl class="flex flex-col gap-2">
            {#each system.capValues as cap (cap.key)}
              <div class="flex justify-between">
                <dt class="text-sm text-muted-foreground">{cap.label}</dt>
                <dd class="text-sm font-medium text-foreground">
                  {cap.value}
                  {cap.unit}
                </dd>
              </div>
            {/each}
          </dl>

          {#if system.upgradeBlockedReason}
            <p class="text-sm text-muted-foreground">
              {system.upgradeBlockedReason}
            </p>
          {/if}
        </Card.Content>
        <Card.Footer>
          {#if system.canUpgrade}
            <Button onclick={() => handleUpgrade(system.id)} disabled={isUpgrading} class="w-full">
              {isUpgrading ? 'Upgrading...' : `Upgrade (${system.upgradeCostMaterials} Materials)`}
            </Button>
          {:else}
            <Button variant="outline" disabled class="w-full">
              {system.upgradeCostMaterials
                ? `${system.upgradeCostMaterials} Materials`
                : 'Max Level'}
            </Button>
          {/if}
        </Card.Footer>
      </Card.Root>
    {/each}
  </div>
{/if}
