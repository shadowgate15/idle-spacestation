import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createResourcesPanelState } from './resources-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(materials: number, data: number) {
  return {
    ...baseSnapshot,
    resources: {
      ...baseSnapshot.resources,
      materials,
      data,
    },
  };
}

describe('createResourcesPanelState', () => {
  it('initializes drafts from null snapshot', () => {
    const state = createResourcesPanelState(null, {
      applyResources: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.materialsDraft).toBe(0);
    expect(state.dataDraft).toBe(0);
  });

  it('syncs and applies successful updates', async () => {
    const nextSnapshot = createSnapshot(8, 13);
    const state = createResourcesPanelState(createSnapshot(1, 2), {
      applyResources: vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot }),
    });

    state.materialsDraft = 8;
    state.dataDraft = 13;
    await state.apply();

    expect(state.snapshot?.resources.materials).toBe(8);
    expect(state.snapshot?.resources.data).toBe(13);
    expect(state.errorMessage).toBeNull();
  });

  it('stores failure reason codes', async () => {
    const state = createResourcesPanelState(createSnapshot(1, 2), {
      applyResources: vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'invalid_state',
        snapshot: createSnapshot(1, 2),
      }),
    });

    await state.apply();

    expect(state.errorMessage).toBe('invalid_state');
  });
});
