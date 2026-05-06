import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createServicesPanelState } from './services-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot() {
  return {
    ...baseSnapshot,
    services: baseSnapshot.services.map((service, index) => ({
      ...service,
      desiredActive: index % 2 === 0,
      assignedCrew: index,
      priority: index + 1,
      isActive: index % 3 === 0,
      isPaused: index % 3 === 1,
      pauseReason: index % 3 === 1 ? ('crew' as const) : null,
    })),
  };
}

function createUpdatedSnapshot() {
  return {
    ...baseSnapshot,
    services: baseSnapshot.services.map((service, index, services) => ({
      ...service,
      desiredActive: index % 2 !== 0,
      assignedCrew: services.length - index,
      priority: services.length - index,
      isActive: index % 2 === 0,
      isPaused: false,
      pauseReason: null,
    })),
  };
}

describe('createServicesPanelState', () => {
  it('initializes drafts from null snapshot', () => {
    const state = createServicesPanelState(null, {
      applyServices: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.drafts).toEqual([]);
  });

  it('initializes drafts from a real snapshot', () => {
    const snapshot = createSnapshot();
    const state = createServicesPanelState(snapshot, {
      applyServices: vi.fn(),
    });

    expect(state.snapshot?.services).toEqual(snapshot.services);
    expect(
      state.drafts.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    ).toEqual(
      snapshot.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    );
  });

  it('applies successful updates', async () => {
    const initialSnapshot = createSnapshot();
    const nextSnapshot = createUpdatedSnapshot();
    const applyServices = vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot });
    const state = createServicesPanelState(initialSnapshot, {
      applyServices,
    });

    for (const [index, draft] of state.drafts.entries()) {
      const nextService = nextSnapshot.services[index];
      draft.desiredActive = nextService.desiredActive;
      draft.assignedCrew = nextService.assignedCrew;
      draft.priority = nextService.priority;
    }

    await state.apply();

    expect(applyServices).toHaveBeenCalledWith({
      services: nextSnapshot.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    });
    expect(state.snapshot?.services).toEqual(nextSnapshot.services);
    expect(
      state.drafts.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    ).toEqual(
      nextSnapshot.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    );
    expect(state.errorMessage).toBeNull();
  });

  it('stores failure reason codes', async () => {
    const snapshot = createSnapshot();
    const state = createServicesPanelState(snapshot, {
      applyServices: vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'constraint_violation',
        snapshot,
      }),
    });

    await state.apply();

    expect(state.errorMessage).toBe('constraint_violation');
  });

  it('polling does not overwrite typed assignedCrew for service A', () => {
    const state = createServicesPanelState(createSnapshot(), {
      applyServices: vi.fn(),
    });

    state.drafts[0].assignedCrew = 99;
    state.sync(createUpdatedSnapshot());

    expect(state.drafts[0].assignedCrew).toBe(99);
    expect(state.snapshot?.services[0].assignedCrew).toBe(
      createUpdatedSnapshot().services[0].assignedCrew,
    );
  });

  it('polling does not overwrite typed priority for service B', () => {
    const state = createServicesPanelState(createSnapshot(), {
      applyServices: vi.fn(),
    });

    state.drafts[1].priority = 42;
    state.sync(createUpdatedSnapshot());

    expect(state.drafts[1].priority).toBe(42);
    expect(state.snapshot?.services[1].priority).toBe(
      createUpdatedSnapshot().services[1].priority,
    );
  });

  it('polling does not toggle desiredActive checkbox state', () => {
    const initialSnapshot = createSnapshot();
    const state = createServicesPanelState(initialSnapshot, {
      applyServices: vi.fn(),
    });

    const initialDesired = state.drafts[0].desiredActive;
    state.drafts[0].desiredActive = !initialDesired;

    state.sync(initialSnapshot);

    expect(state.drafts[0].desiredActive).toBe(!initialDesired);
  });

  it('keeps drafts array reference stable across polling', () => {
    const state = createServicesPanelState(createSnapshot(), {
      applyServices: vi.fn(),
    });
    const drafts = state.drafts;

    state.sync(createUpdatedSnapshot());

    expect(state.drafts).toBe(drafts);
  });

  it('apply success mutates drafts in place and clears isDirty', async () => {
    const initialSnapshot = createSnapshot();
    const nextSnapshot = createUpdatedSnapshot();
    const state = createServicesPanelState(initialSnapshot, {
      applyServices: vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot }),
    });
    const drafts = state.drafts;

    for (const [index, draft] of state.drafts.entries()) {
      const nextService = nextSnapshot.services[index];
      draft.desiredActive = nextService.desiredActive;
      draft.assignedCrew = nextService.assignedCrew;
      draft.priority = nextService.priority;
    }

    await state.apply();

    expect(state.drafts).toBe(drafts);
    expect(
      state.drafts.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    ).toEqual(
      nextSnapshot.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
        id,
        desiredActive,
        assignedCrew,
        priority,
      })),
    );
    expect(state.isDirty).toBe(false);
  });
});
