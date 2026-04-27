<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { createGameGateway } from '$lib/game/api';
  import type { GameSnapshot } from '$lib/game/api/types';
  import { cn } from '$lib/utils';
  import { createSessionPanelState } from './session-panel-state.svelte';

  let {
    snapshot,
    gateway,
  }: {
    snapshot: GameSnapshot | null;
    gateway: ReturnType<typeof createGameGateway>;
  } = $props();

  const state = createSessionPanelState(null, {
    advanceTicks: (input) => gateway.advanceTicks(input),
    resetToStarter: (input) => gateway.resetToStarter(input),
  });

  $effect(() => {
    untrack(() => state.sync(snapshot));
  });
</script>

<div data-testid="devtools-session-panel" class="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
  <div class="mb-3">
    <h4 class="text-sm font-semibold text-zinc-100">Session</h4>
    <p class="text-xs text-zinc-500">Advance the active session or reset back to the starter state.</p>
  </div>

  <div class="grid gap-3">
    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">
        Current tick: {state.snapshot?.meta.tickCount ?? '—'}
      </div>

      <div class="grid grid-cols-[1fr_auto] items-end gap-3">
        <label class="grid gap-1.5 text-xs text-zinc-400">
          <span>Advance ticks</span>
          <Input
            data-testid="devtools-advance-ticks-input"
            type="number"
            min="1"
            max="240"
            step="1"
            value={state.advanceCount}
            onchange={(event) => state.setAdvanceCount(event.currentTarget.valueAsNumber)}
            disabled={state.isBusy}
            class="font-mono text-zinc-100"
          />
        </label>

        <Button
          data-testid="devtools-advance-ticks-btn"
          size="sm"
          onclick={() => state.advance()}
          disabled={!state.snapshot || state.isBusy}
        >
          {state.isAdvancing ? 'Advancing…' : 'Advance'}
        </Button>
      </div>
    </div>

    <div class="grid gap-2 rounded-md border border-zinc-800 bg-zinc-900/60 p-2">
      <div class="text-xs font-semibold text-zinc-200">Reset to starter</div>
      <div class="text-[0.625rem] font-mono uppercase tracking-wide text-zinc-500">Restore the baseline starter run.</div>

      {#if state.isConfirmingReset}
        <div class="flex flex-wrap items-center gap-2 text-xs text-zinc-400">
          <span>Are you sure?</span>
          <Button
            data-testid="devtools-reset-confirm-btn"
            size="sm"
            variant="destructive"
            onclick={() => state.confirmReset()}
            disabled={!state.snapshot || state.isBusy}
          >
            {state.isResetting ? 'Resetting…' : 'Confirm reset'}
          </Button>
          <Button size="sm" variant="ghost" onclick={() => state.cancelResetConfirmation()} disabled={state.isBusy}>
            Cancel
          </Button>
        </div>
      {:else}
        <div>
          <Button
            data-testid="devtools-reset-to-starter-btn"
            size="sm"
            variant="destructive"
            onclick={() => state.requestResetConfirmation()}
            disabled={!state.snapshot || state.isBusy}
          >
            Reset
          </Button>
        </div>
      {/if}
    </div>
  </div>

  <div class="mt-3">
    <p
      data-testid="devtools-session-error"
      class={cn('min-h-4 text-xs font-medium', state.errorMessage ? 'text-amber-400' : 'text-zinc-600')}
    >
      {state.errorMessage ?? ''}
    </p>
  </div>
</div>
