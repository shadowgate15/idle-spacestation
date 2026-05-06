import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createSessionPanelState } from './session-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(tickCount = 12) {
  return {
    ...baseSnapshot,
    meta: {
      ...baseSnapshot.meta,
      tickCount,
    },
  };
}

describe('createSessionPanelState', () => {
  it('initializes from null snapshot', () => {
    const state = createSessionPanelState(null, {
      advanceTicks: vi.fn(),
      resetToStarter: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.advanceCount).toBe(1);
    expect(state.isConfirmingReset).toBe(false);
  });

  it('initializes from a real snapshot', () => {
    const snapshot = createSnapshot(48);
    const state = createSessionPanelState(snapshot, {
      advanceTicks: vi.fn(),
      resetToStarter: vi.fn(),
    });

    expect(state.snapshot?.meta.tickCount).toBe(48);
    expect(state.advanceCount).toBe(1);
  });

  it('advances ticks through the gateway', async () => {
    const nextSnapshot = createSnapshot(64);
    const advanceTicks = vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot });
    const state = createSessionPanelState(createSnapshot(24), {
      advanceTicks,
      resetToStarter: vi.fn(),
    });

    state.setAdvanceCount(40);
    await state.advance();

    expect(advanceTicks).toHaveBeenCalledWith({ count: 40 });
    expect(state.snapshot?.meta.tickCount).toBe(64);
    expect(state.errorMessage).toBeNull();
  });

  it('handles reset confirmation flow', async () => {
    const resetSnapshot = createSnapshot(0);
    const resetToStarter = vi.fn().mockResolvedValue({ ok: true, snapshot: resetSnapshot });
    const state = createSessionPanelState(createSnapshot(24), {
      advanceTicks: vi.fn(),
      resetToStarter,
    });

    state.requestResetConfirmation();
    expect(state.isConfirmingReset).toBe(true);

    await state.confirmReset();

    expect(resetToStarter).toHaveBeenCalledWith({});
    expect(state.snapshot?.meta.tickCount).toBe(0);
    expect(state.isConfirmingReset).toBe(false);
    expect(state.errorMessage).toBeNull();
  });
});
