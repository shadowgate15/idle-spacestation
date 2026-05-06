import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createSystemsPanelState } from './systems-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(levels: number[]) {
  return {
    ...baseSnapshot,
    systems: baseSnapshot.systems.map((system, index) => ({
      ...system,
      level: levels[index] ?? system.level,
    })),
  };
}

describe('createSystemsPanelState', () => {
  it('initializes drafts from null snapshot', () => {
    const state = createSystemsPanelState(null, {
      applySystems: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.drafts).toEqual([]);
  });

  it('initializes drafts from a real snapshot', () => {
    const snapshot = createSnapshot([1, 2, 3, 4]);
    const state = createSystemsPanelState(snapshot, {
      applySystems: vi.fn(),
    });

    expect(state.snapshot?.systems).toEqual(snapshot.systems);
    expect(state.drafts.map(({ id, level }) => ({ id, level }))).toEqual(snapshot.systems);
  });

  it('applies successful updates', async () => {
    const nextSnapshot = createSnapshot([4, 3, 2, 1]);
    const applySystems = vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot });
    const state = createSystemsPanelState(createSnapshot([1, 1, 1, 1]), {
      applySystems,
    });

    state.drafts[0].level = 4;
    state.drafts[1].level = 3;
    state.drafts[2].level = 2;
    state.drafts[3].level = 1;

    await state.apply();

    expect(applySystems).toHaveBeenCalledWith({ systems: nextSnapshot.systems });
    expect(state.snapshot?.systems).toEqual(nextSnapshot.systems);
    expect(state.drafts.map(({ id, level }) => ({ id, level }))).toEqual(nextSnapshot.systems);
    expect(state.errorMessage).toBeNull();
  });

  it('stores failure reason codes', async () => {
    const snapshot = createSnapshot([1, 2, 3, 4]);
    const state = createSystemsPanelState(snapshot, {
      applySystems: vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'invalid_state',
        snapshot,
      }),
    });

    await state.apply();

    expect(state.errorMessage).toBe('invalid_state');
  });
});
