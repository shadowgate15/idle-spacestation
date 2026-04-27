<script lang="ts">
  import type { GameSnapshot } from '$lib/game/api/types';

  let { snapshot, onClose }: { snapshot: GameSnapshot | null; onClose: () => void } = $props();
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
  </section>

  <section data-testid="devtools-systems-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Systems</h3>
  </section>

  <section data-testid="devtools-services-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Services</h3>
  </section>

  <section data-testid="devtools-progression-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Progression</h3>
  </section>

  <section data-testid="devtools-session-section">
    <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-400">Session</h3>
  </section>
</div>
