<script lang="ts">
  import { gameState } from '$lib/game/api/state.svelte';
  import { gameGateway } from '$lib/game/api';
  import type { GameSnapshot, SystemId } from '$lib/game/api/types';
  import SnapshotGuard from '$lib/components/SnapshotGuard.svelte';
  import * as Card from '$lib/components/ui/card';
  import Button from '$lib/components/ui/button/button.svelte';
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

  async function handleUpgrade(systemId: SystemId) {
    await runInflightAction(systemId, () => gameGateway.upgradeSystem({ systemId }));
  }
</script>

<SnapshotGuard loadingMessage="Loading station systems...">
  {#snippet children(snapshot)}
    {@const systems = snapshot.routes.systems}
    <section data-testid="systems-header" class="mb-8">
      <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Station Systems</h2>
      <p class="text-muted-foreground">
        Upgrade to expand station capabilities. Each system gates specific limits.
      </p>
    </section>

    <div class="grid gap-6 lg:grid-cols-2">
      {#each systems.systems as system (system.id)}
        {@const isUpgrading = inflight.has(system.id)}
        <Card.Root>
          <Card.Header>
            <Card.Title>{system.name}</Card.Title>
            <Card.Description>{system.description}</Card.Description>
          </Card.Header>
          <Card.Content class="flex flex-col gap-4">
            <dl class="grid grid-cols-[auto_1fr_auto] items-center gap-x-6 gap-y-2">
              <StatRow kind="ratio" label="Level" used={system.level} total={system.maxLevel} />
              {#each system.capValues as cap (cap.key)}
                <StatRow kind="scalar" label={cap.label} value={cap.value} unit={cap.unit} />
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
              <Button
                onclick={() => handleUpgrade(system.id)}
                disabled={isUpgrading}
                class="w-full"
              >
                {isUpgrading
                  ? 'Upgrading...'
                  : `Upgrade (${system.upgradeCostMaterials} Materials)`}
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
  {/snippet}
</SnapshotGuard>
