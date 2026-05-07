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

  it('polling does not overwrite typed level draft for system A', () => {
    const state = createSystemsPanelState(createSnapshot([1, 1, 1, 1]), {
      applySystems: vi.fn(),
    });

    state.drafts[0].level = 4;
    state.sync(createSnapshot([2, 2, 2, 2]));

    expect(state.drafts[0].level).toBe(4);
    expect(state.snapshot?.systems[0].level).toBe(2);
  });

  it('keeps drafts array reference stable across polling', () => {
    const state = createSystemsPanelState(createSnapshot([1, 1, 1, 1]), {
      applySystems: vi.fn(),
    });
    const drafts = state.drafts;

    state.sync(createSnapshot([2, 2, 2, 2]));

    expect(state.drafts).toBe(drafts);
  });

  it('apply success mutates drafts in place to response snapshot levels', async () => {
    const nextSnapshot = createSnapshot([4, 3, 2, 1]);
    const state = createSystemsPanelState(createSnapshot([1, 1, 1, 1]), {
      applySystems: vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot }),
    });
    const drafts = state.drafts;

    state.drafts[0].level = 4;
    state.drafts[1].level = 3;
    state.drafts[2].level = 2;
    state.drafts[3].level = 1;

    await state.apply();

    expect(state.drafts).toBe(drafts);
    expect(state.drafts.map(({ id, level }) => ({ id, level }))).toEqual(nextSnapshot.systems);
    expect(state.isDirty).toBe(false);
  });

  it('does not flicker isDirty on polling when drafts are unchanged', () => {
    const state = createSystemsPanelState(createSnapshot([1, 2, 3, 4]), {
      applySystems: vi.fn(),
    });

    expect(state.isDirty).toBe(false);

    state.sync(createSnapshot([4, 3, 2, 1]));

    expect(state.isDirty).toBe(false);
    expect(state.drafts.map(({ id, level }) => ({ id, level }))).toEqual(
      createSnapshot([1, 2, 3, 4]).systems,
    );
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
