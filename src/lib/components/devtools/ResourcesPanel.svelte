<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot } from '$lib/game/api/types';
  import { cn } from '$lib/utils';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const MATERIALS_MIN = 0;
  const MATERIALS_MAX = 99999;
  const DATA_MIN = 0;
  const DATA_MAX = 99999;

  let panelSnapshot = $state<GameSnapshot | null>(null);
  let materialsDraft = $state<number | undefined>(0);
  let dataDraft = $state<number | undefined>(0);
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);
  let lastSnapshot: GameSnapshot | null = null;

  $effect(() => {
    const previousMaterials = lastSnapshot?.resources.materials ?? 0;
    const previousData = lastSnapshot?.resources.data ?? 0;
    const wasDirty = untrack(() => materialsDraft) !== previousMaterials || untrack(() => dataDraft) !== previousData;

    lastSnapshot = snapshot;
    panelSnapshot = snapshot;

    if (!wasDirty) {
      materialsDraft = snapshot?.resources.materials ?? 0;
      dataDraft = snapshot?.resources.data ?? 0;

      if (snapshot) {
        errorMessage = null;
      }
    }
  });

  function isInRange(value: number | undefined, min: number, max: number): value is number {
    return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
  }

  async function applyDraft() {
    if (
      !isInRange(materialsDraft, MATERIALS_MIN, MATERIALS_MAX) ||
      !isInRange(dataDraft, DATA_MIN, DATA_MAX)
    ) {
      errorMessage = 'invalid_range';
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyResources({
        materials: materialsDraft,
        data: dataDraft,
      });

      lastSnapshot = response.snapshot;
      panelSnapshot = response.snapshot;

      if (response.ok) {
        materialsDraft = response.snapshot.resources.materials;
        dataDraft = response.snapshot.resources.data;
        return;
      }

      errorMessage = response.reasonCode;
    } finally {
      isApplying = false;
    }
  }
</script>

<div data-testid="devtools-resources-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3 flex items-start justify-between gap-3">
    <div>
      <h4 class="text-sm font-semibold text-zinc-100">Resources</h4>
      <p class="text-xs text-zinc-500">Stage materials and data values, then apply them.</p>
    </div>

    <div class="text-right text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
      <div>Power</div>
      <div>{panelSnapshot ? `${panelSnapshot.resources.power.available} avail` : 'offline'}</div>
    </div>
  </div>

  <div class="grid gap-3">
    <label class="grid gap-1.5 text-xs text-zinc-400">
      <span>Materials</span>
      <Input
        data-testid="devtools-materials-input"
        type="number"
        min="0"
        max="99999"
        step="1"
        bind:value={materialsDraft}
        disabled={!panelSnapshot || isApplying}
        class="font-mono text-zinc-100"
      />
    </label>

    <label class="grid gap-1.5 text-xs text-zinc-400">
      <span>Data</span>
      <Input
        data-testid="devtools-data-input"
        type="number"
        min="0"
        max="99999"
        step="1"
        bind:value={dataDraft}
        disabled={!panelSnapshot || isApplying}
        class="font-mono text-zinc-100"
      />
    </label>

    <div class="rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="mb-2 text-[0.625rem] font-semibold uppercase tracking-wide text-zinc-500">
        Power (read only)
      </div>

      <div class="grid grid-cols-3 gap-2 text-xs font-mono text-zinc-300">
        <div class="grid gap-1">
          <span class="text-zinc-500">Generated</span>
          <span>{panelSnapshot ? panelSnapshot.resources.power.generated : '—'}</span>
        </div>
        <div class="grid gap-1">
          <span class="text-zinc-500">Reserved</span>
          <span>{panelSnapshot ? panelSnapshot.resources.power.reserved : '—'}</span>
        </div>
        <div class="grid gap-1">
          <span class="text-zinc-500">Available</span>
          <span>{panelSnapshot ? panelSnapshot.resources.power.available : '—'}</span>
        </div>
      </div>
    </div>
  </div>

  <div class="mt-3 flex items-center justify-between gap-3">
    <p
      data-testid="devtools-resources-error"
      class={cn('min-h-4 text-xs font-medium', errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {errorMessage ?? ''}
    </p>

    <Button
      data-testid="devtools-resources-apply"
      size="sm"
      onclick={applyDraft}
      disabled={!panelSnapshot || isApplying}
    >
      {isApplying ? 'Applying…' : 'Apply'}
    </Button>
  </div>
</div>
