import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createApplyPanelState, type ApplyResponse } from './_create-apply-panel-state.svelte';
import type { GameSnapshot } from '$lib/game/api/types';

type ResourceDraft = {
  materials: number | undefined;
  data: number | undefined;
};

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

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });

  return { promise, resolve };
}

function createResourcePanelState(
  initialSnapshot: GameSnapshot | null,
  applyToGateway = vi
    .fn<(draft: ResourceDraft) => Promise<ApplyResponse>>()
    .mockResolvedValue({ ok: true, snapshot: createSnapshot(1, 2) }),
) {
  return {
    applyToGateway,
    state: createApplyPanelState(initialSnapshot, {
      seedDraft: (snapshot) => ({
        materials: snapshot?.resources.materials ?? 0,
        data: snapshot?.resources.data ?? 0,
      }),
      cloneDraft: (draft) => ({ ...draft }),
      isDirty: (draft, baseline) =>
        draft.materials !== baseline.materials || draft.data !== baseline.data,
      isValid: (draft) => isInRange(draft.materials, 0, 99999) && isInRange(draft.data, 0, 99999),
      applyToGateway,
    }),
  };
}

describe('createApplyPanelState', () => {
  it('initializes drafts from the initial snapshot when present', () => {
    const { state } = createResourcePanelState(createSnapshot(3, 5));

    expect(state.snapshot?.resources.materials).toBe(3);
    expect(state.draft).toEqual({ materials: 3, data: 5 });
    expect(state.isDirty).toBe(false);
  });

  it('sync seeds drafts only on the first non-null snapshot', () => {
    const { state } = createResourcePanelState(null);

    state.sync(null);
    expect(state.snapshot).toBeNull();
    expect(state.draft).toEqual({ materials: 0, data: 0 });

    state.sync(createSnapshot(3, 5));
    expect(state.draft).toEqual({ materials: 3, data: 5 });
    expect(state.errorMessage).toBeNull();

    state.draft.materials = 8;
    state.sync(createSnapshot(13, 21));

    expect(state.snapshot?.resources.materials).toBe(13);
    expect(state.draft).toEqual({ materials: 8, data: 5 });
  });

  it('tracks dirty state from seed through mutation and successful apply', async () => {
    const responseSnapshot = createSnapshot(8, 13);
    const { state } = createResourcePanelState(
      createSnapshot(3, 5),
      vi.fn().mockResolvedValue({ ok: true, snapshot: responseSnapshot }),
    );

    expect(state.isDirty).toBe(false);

    state.draft.materials = 8;
    expect(state.isDirty).toBe(true);

    await state.apply();

    expect(state.draft).toEqual({ materials: 8, data: 13 });
    expect(state.isDirty).toBe(false);
  });

  it('sets invalid_range, skips the gateway, and reverts the draft to the baseline for invalid drafts', async () => {
    const applyToGateway = vi.fn<(draft: ResourceDraft) => Promise<ApplyResponse>>();
    const { state } = createResourcePanelState(createSnapshot(3, 5), applyToGateway);

    state.draft.materials = -1;
    state.draft.data = 999999;
    await state.apply();

    expect(state.errorMessage).toBe('invalid_range');
    expect(applyToGateway).not.toHaveBeenCalled();
    expect(state.draft).toEqual({ materials: 3, data: 5 });
    expect(state.isDirty).toBe(false);
  });

  it('sets isApplying during a valid gateway call and clears it in finally', async () => {
    const deferred = createDeferred<ApplyResponse>();
    const applyToGateway = vi.fn<(draft: ResourceDraft) => Promise<ApplyResponse>>(
      () => deferred.promise,
    );
    const { state } = createResourcePanelState(createSnapshot(3, 5), applyToGateway);

    state.draft.materials = 8;
    const applyPromise = state.apply();

    expect(applyToGateway).toHaveBeenCalledWith({ materials: 8, data: 5 });
    expect(state.isApplying).toBe(true);

    deferred.resolve({ ok: true, snapshot: createSnapshot(8, 5) });
    await applyPromise;

    expect(state.isApplying).toBe(false);
  });

  it('clears isApplying when a valid gateway call rejects', async () => {
    const applyToGateway = vi
      .fn<(draft: ResourceDraft) => Promise<ApplyResponse>>()
      .mockRejectedValue(new Error('network failed'));
    const { state } = createResourcePanelState(createSnapshot(3, 5), applyToGateway);

    state.draft.data = 8;
    await expect(state.apply()).rejects.toThrow('network failed');

    expect(state.isApplying).toBe(false);
  });

  it('reseeds drafts and clears errors after a successful apply', async () => {
    const { state } = createResourcePanelState(
      createSnapshot(3, 5),
      vi.fn().mockResolvedValue({ ok: true, snapshot: createSnapshot(8, 13) }),
    );

    state.draft.materials = -1;
    await state.apply();
    expect(state.errorMessage).toBe('invalid_range');

    state.draft.materials = 8;
    state.draft.data = 13;
    await state.apply();

    expect(state.snapshot?.resources.materials).toBe(8);
    expect(state.draft).toEqual({ materials: 8, data: 13 });
    expect(state.errorMessage).toBeNull();
    expect(state.isDirty).toBe(false);
  });

  it('stores rejection reason without reseeding drafts', async () => {
    const { state } = createResourcePanelState(
      createSnapshot(3, 5),
      vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'invalid_state',
        snapshot: createSnapshot(21, 34),
      }),
    );

    state.draft.materials = 8;
    await state.apply();

    expect(state.snapshot?.resources.materials).toBe(21);
    expect(state.draft).toEqual({ materials: 8, data: 5 });
    expect(state.errorMessage).toBe('invalid_state');
    expect(state.isDirty).toBe(true);
  });
});

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
