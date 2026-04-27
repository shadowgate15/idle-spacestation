import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createCrewPanelState } from './crew-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(crewTotal: number) {
  return {
    ...baseSnapshot,
    resources: {
      ...baseSnapshot.resources,
      crew: {
        ...baseSnapshot.resources.crew,
        total: crewTotal,
        available: Math.max(crewTotal - baseSnapshot.resources.crew.assigned, 0),
      },
    },
  };
}

describe('createCrewPanelState', () => {
  it('initializes draft from null snapshot', () => {
    const state = createCrewPanelState(null, {
      applyCrew: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.crewTotalDraft).toBe(0);
  });

  it('syncs and applies successful updates', async () => {
    const nextSnapshot = createSnapshot(12);
    const state = createCrewPanelState(createSnapshot(6), {
      applyCrew: vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot }),
    });

    state.crewTotalDraft = 12;
    await state.apply();

    expect(state.snapshot?.resources.crew.total).toBe(12);
    expect(state.errorMessage).toBeNull();
  });

  it('stores failure reason codes', async () => {
    const state = createCrewPanelState(createSnapshot(6), {
      applyCrew: vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'constraint_violation',
        snapshot: createSnapshot(6),
      }),
    });

    await state.apply();

    expect(state.errorMessage).toBe('constraint_violation');
  });
});
