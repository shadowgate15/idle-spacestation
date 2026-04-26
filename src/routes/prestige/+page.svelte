<script lang="ts">
  import { onMount } from 'svelte';
  import { gameGateway } from '$lib/game/api';
  import type { PrestigeViewModel } from '$lib/game/api/types';

  let prestige = $state<PrestigeViewModel | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let showResetConsequences = $state(false);
  let confirmPrestige = $state(false);

  onMount(async () => {
    try {
      const snapshot = await gameGateway.getSnapshot();
      prestige = snapshot.routes.prestige;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load prestige data';
    } finally {
      loading = false;
    }
  });

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function getReasonLabel(code: string): string {
    switch (code) {
      case 'station-tier-below-four':
        return 'Station must reach Tier 4';
      case 'needs-two-non-starter-planets':
        return 'Must discover 2 additional planets';
      case 'unstable-net-power':
        return 'Need 300s stable power (all systems running)';
      default:
        return code;
    }
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-12">
    <p class="text-muted-foreground">Loading prestige data...</p>
  </div>
{:else if error}
  <div class="rounded-lg border border-rose-500 bg-rose-950/30 p-4">
    <p class="text-rose-200">{error}</p>
  </div>
{:else if prestige}
  <section data-testid="prestige-hero" class="mb-8">
    <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Prestige Operations</h2>
    <p class="mb-4 text-muted-foreground">Ascend to a new era with accumulated knowledge.</p>
  </section>

  <section data-testid="eligibility-panel" class="mb-8 rounded-lg border border-border bg-card p-4">
    <h3 class="mb-3 text-base font-semibold text-foreground">Eligibility Status</h3>

    {#if prestige.eligibility.eligible}
      <div class="mb-4 rounded-lg border border-emerald-500 bg-emerald-950/30 p-4">
        <p class="text-emerald-200">Prestige is available!</p>
        <p class="mt-1 text-sm text-muted-foreground">
          {prestige.eligibility.summary}
        </p>
      </div>
    {:else}
      <div class="mb-4 rounded-lg border border-rose-500 bg-rose-950/30 p-4">
        <p class="text-rose-200">Prestige requirements not met</p>
      </div>

      <ul class="space-y-2 text-sm text-muted-foreground">
        {#each prestige.eligibility.reasonCodes as code (code)}
          <li class="flex items-center gap-2">
            <span class="text-rose-400">✗</span>
            <span>{getReasonLabel(code)}</span>
          </li>
        {/each}
      </ul>
    {/if}

    <div class="mt-4 flex items-center gap-4">
      <div class="flex flex-col">
        <dt class="text-xs tracking-wide text-muted-foreground uppercase">Stable Power Time</dt>
        <dd class="text-lg font-bold text-foreground">
          {formatTime(prestige.eligibility.stablePowerSeconds)} / {formatTime(
            prestige.eligibility.requiredStablePowerSeconds,
          )}
        </dd>
      </div>
    </div>
  </section>

  <section
    data-testid="doctrine-fragments"
    class="mb-8 rounded-lg border border-border bg-card p-4"
  >
    <h3 class="mb-3 text-base font-semibold text-foreground">Doctrine Fragments</h3>
    <p class="text-2xl font-bold text-foreground">
      {prestige.doctrineFragments}
    </p>
  </section>

  {#if prestige.unlockedDoctrines.length > 0}
    <section
      data-testid="unlocked-doctrines"
      class="mb-8 rounded-lg border border-border bg-card p-4"
    >
      <h3 class="mb-3 text-base font-semibold text-foreground">Unlocked Doctrines</h3>
      <ul class="space-y-2">
        {#each prestige.unlockedDoctrines as doctrine (doctrine.id)}
          <li class="rounded border border-border bg-card/50 p-3">
            <h4 class="font-semibold text-foreground">{doctrine.name}</h4>
            <p class="text-sm text-muted-foreground">{doctrine.description}</p>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if prestige.purchaseOptions.length > 0 && prestige.doctrineFragments > 0}
    <section
      data-testid="doctrine-purchase"
      class="mb-8 rounded-lg border border-border bg-card p-4"
    >
      <h3 class="mb-3 text-base font-semibold text-foreground">Available Doctrines</h3>
      <ul class="space-y-2">
        {#each prestige.purchaseOptions as option (option.id)}
          <li class="flex items-center justify-between rounded border border-border bg-card/50 p-3">
            <div>
              <h4 class="font-semibold text-foreground">{option.name}</h4>
              <p class="text-sm text-muted-foreground">{option.description}</p>
              {#if option.blockedReason}
                <p class="mt-1 text-xs text-rose-400">
                  {option.blockedReason === 'already-unlocked'
                    ? 'Already owned'
                    : `Requires ${option.costFragments} fragment${option.costFragments !== 1 ? 's' : ''}`}
                </p>
              {/if}
            </div>
            <span class="text-sm text-muted-foreground">
              {option.costFragments} fragment
            </span>
          </li>
        {/each}
      </ul>
    </section>
  {:else if prestige.purchaseOptions.length > 0}
    <section
      data-testid="doctrine-purchase-locked"
      class="mb-8 rounded-lg border border-border bg-card p-4"
    >
      <h3 class="mb-3 text-base font-semibold text-foreground">Available Doctrines</h3>
      <p class="text-sm text-muted-foreground">No fragments available to purchase doctrines.</p>
    </section>
  {/if}

  <section
    data-testid="reset-consequences"
    class="mb-8 rounded-lg border border-border bg-card p-4"
  >
    <h3 class="mb-3 text-base font-semibold text-foreground">Reset Consequences</h3>

    {#if !showResetConsequences}
      <button
        type="button"
        class="text-sm text-muted-foreground transition-colors hover:text-foreground"
        onclick={() => (showResetConsequences = true)}
      >
        Show what resets vs persists
      </button>
    {:else}
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border text-left">
            <th class="pb-2 font-semibold text-foreground">Category</th>
            <th class="pb-2 font-semibold text-foreground">Outcome</th>
            <th class="pb-2 font-semibold text-foreground">Details</th>
          </tr>
        </thead>
        <tbody>
          {#each prestige.resetConsequences as item (item.label)}
            <tr class="border-b border-border">
              <td class="py-2 text-foreground">{item.label}</td>
              <td class="py-2">
                <span
                  class:text-emerald-400={item.outcome === 'retain'}
                  class:text-rose-400={item.outcome === 'reset'}
                >
                  {item.outcome === 'retain' ? 'Kept' : 'Reset'}
                </span>
              </td>
              <td class="py-2 text-muted-foreground">{item.summary}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  {#if prestige.eligibility.eligible}
    <section data-testid="prestige-action" class="rounded-lg border border-border bg-card p-4">
      {#if !confirmPrestige}
        <button
          type="button"
          class="rounded-lg bg-foreground px-4 py-2 text-background transition-colors hover:bg-muted-foreground"
          onclick={() => (confirmPrestige = true)}
        >
          Begin Prestige
        </button>
      {:else}
        <div class="rounded-lg border border-amber-500 bg-amber-950/30 p-4">
          <p class="mb-4 text-amber-200">
            This action cannot be undone. Your current run progress will be reset.
          </p>
          <div class="flex gap-4">
            <button
              type="button"
              class="rounded-lg bg-rose-600 px-4 py-2 text-white transition-colors hover:bg-rose-700"
              onclick={() => {
                confirmPrestige = false;
                showResetConsequences = false;
              }}
            >
              Cancel
            </button>
            <button
              type="button"
              class="rounded-lg bg-emerald-600 px-4 py-2 text-white transition-colors hover:bg-emerald-700"
              disabled
            >
              Confirm Prestige
            </button>
          </div>
        </div>
      {/if}
    </section>
  {/if}
{/if}
