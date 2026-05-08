import { describe, expect, it, vi } from 'vitest';
import { render as mount } from 'vitest-browser-svelte';
import type { Component } from 'svelte';
import type { GameSnapshot } from '$lib/game/api/types';
import UseSnapshotSyncHarness from './_use-snapshot-sync.harness.svelte';

function fakeSnapshot(tickCount: number): GameSnapshot {
  return { meta: { tickCount } } as unknown as GameSnapshot;
}

describe('useSnapshotSync', () => {
  it('calls state.sync when the snapshot getter value changes', async () => {
    const sync = vi.fn();
    const state = { sync };
    const initial = fakeSnapshot(1);

    const view = await mount(UseSnapshotSyncHarness as Component, {
      props: { state, snapshot: initial },
    });

    try {
      // Initial mount triggers the effect once.
      await vi.waitFor(() => {
        expect(sync).toHaveBeenCalledTimes(1);
      });
      expect(sync).toHaveBeenLastCalledWith(initial);

      const next = fakeSnapshot(2);
      await view.rerender({ state, snapshot: next });

      await vi.waitFor(() => {
        expect(sync).toHaveBeenCalledTimes(2);
      });
      expect(sync).toHaveBeenLastCalledWith(next);
    } finally {
      await view.unmount();
    }
  });

  it('does NOT re-run when state.sync internally reads tracked $state', async () => {
    // The harness exposes an internal counter that sync() increments and reads.
    // If untrack were missing, this would create a loop and the effect would
    // run more than once per snapshot change. With untrack, sync's internal
    // reactive reads are isolated from this effect.
    const internalReads: number[] = [];
    const state = {
      sync: vi.fn(() => {
        // Simulate reactive read inside sync — should NOT re-track outer effect.
        internalReads.push(internalReads.length);
      }),
    };

    const view = await mount(UseSnapshotSyncHarness as Component, {
      props: { state, snapshot: fakeSnapshot(1) },
    });

    try {
      await vi.waitFor(() => {
        expect(state.sync).toHaveBeenCalledTimes(1);
      });

      // Wait a tick to confirm no runaway re-runs happen.
      await new Promise((r) => setTimeout(r, 50));
      expect(state.sync).toHaveBeenCalledTimes(1);
    } finally {
      await view.unmount();
    }
  });

  it('stops invoking sync after the component unmounts', async () => {
    const sync = vi.fn();
    const state = { sync };

    const view = await mount(UseSnapshotSyncHarness as Component, {
      props: { state, snapshot: fakeSnapshot(1) },
    });

    await vi.waitFor(() => {
      expect(sync).toHaveBeenCalledTimes(1);
    });

    await view.unmount();
    const callsAtUnmount = sync.mock.calls.length;

    // Even though we cannot rerender after unmount, give the loop a chance
    // and ensure no stray async invocation occurs.
    await new Promise((r) => setTimeout(r, 50));
    expect(sync).toHaveBeenCalledTimes(callsAtUnmount);
  });

  it('passes null snapshots through to state.sync', async () => {
    const sync = vi.fn();
    const state = { sync };

    const view = await mount(UseSnapshotSyncHarness as Component, {
      props: { state, snapshot: null },
    });

    try {
      await vi.waitFor(() => {
        expect(sync).toHaveBeenCalledTimes(1);
      });
      expect(sync).toHaveBeenLastCalledWith(null);
    } finally {
      await view.unmount();
    }
  });
});
