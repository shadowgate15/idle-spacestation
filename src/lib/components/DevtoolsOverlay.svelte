<script lang="ts">
  import { createGameGateway } from '$lib/game/api';
  import CrewPanel from '$lib/components/devtools/CrewPanel.svelte';
  import ProgressionPanel from '$lib/components/devtools/ProgressionPanel.svelte';
  import ResourcesPanel from '$lib/components/devtools/ResourcesPanel.svelte';
  import SessionPanel from '$lib/components/devtools/SessionPanel.svelte';
  import ServicesPanel from '$lib/components/devtools/ServicesPanel.svelte';
  import SystemsPanel from '$lib/components/devtools/SystemsPanel.svelte';
  import type { GameSnapshot } from '$lib/game/api/types';

  let {
    snapshot,
    gateway,
    onClose,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
    onClose: () => void;
  } = $props();
</script>

<div
  data-testid="devtools-overlay"
  class="fixed top-0 right-0 z-50 flex h-screen w-96 flex-col gap-4 overflow-y-auto border-l border-zinc-700 bg-zinc-900 p-4"
>
  <div class="flex items-center justify-between">
    <h2 class="text-sm font-semibold text-zinc-100">Dev Inspector</h2>
    <button
      data-testid="devtools-close-btn"
      onclick={onClose}
      class="text-zinc-400 hover:text-zinc-100"
      aria-label="Close dev inspector"
    >
      ✕
    </button>
  </div>

  {#if snapshot}
    <p class="font-mono text-xs text-zinc-500">
      Tick: {snapshot.meta.tickCount} · Tier: {snapshot.run.stationTier}
    </p>
  {/if}

  <section data-testid="devtools-resources-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Resources & Crew</h3>
    <div class="grid gap-3">
      <ResourcesPanel {snapshot} {gateway} />
      <CrewPanel {snapshot} {gateway} />
    </div>
  </section>

  <section data-testid="devtools-systems-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Systems</h3>
    <SystemsPanel {snapshot} {gateway} />
  </section>

  <section data-testid="devtools-services-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Services</h3>
    <ServicesPanel {snapshot} {gateway} />
  </section>

  <section data-testid="devtools-progression-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Progression</h3>
    <ProgressionPanel {snapshot} {gateway} />
  </section>

  <section data-testid="devtools-session-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Session</h3>
    <SessionPanel {snapshot} {gateway} />
  </section>
</div>
