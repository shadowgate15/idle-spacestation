import { describe, expect, it, vi } from 'vitest';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import type { DoctrineId, GameSnapshot, PlanetId } from '$lib/game/api/types';
import { PLANET_IDS } from '$lib/game/api/types';
import { createProgressionPanelState, planetIds } from './progression-panel-state.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(): GameSnapshot {
  return {
    ...baseSnapshot,
    run: {
      ...baseSnapshot.run,
      doctrineFragments: 7,
      doctrineIds: ['efficient-shifts', 'hardened-relays'] satisfies DoctrineId[],
      discoveredPlanetIds: ['solstice-anchor', 'cinder-forge'] satisfies PlanetId[],
      activePlanetId: 'cinder-forge' as const,
      surveyProgress: 600,
    },
  };
}

function createUpdatedSnapshot(): GameSnapshot {
  return {
    ...baseSnapshot,
    run: {
      ...baseSnapshot.run,
      doctrineFragments: 11,
      doctrineIds: [
        'efficient-shifts',
        'deep-survey-protocols',
        'frontier-charters',
      ] satisfies DoctrineId[],
      discoveredPlanetIds: ['solstice-anchor', 'cinder-forge', 'aurora-pier'] satisfies PlanetId[],
      activePlanetId: 'aurora-pier' as const,
      surveyProgress: 1260,
    },
  };
}

describe('createProgressionPanelState', () => {
  it('initializes from null snapshot', () => {
    const state = createProgressionPanelState(null, {
      applyProgression: vi.fn(),
    });

    expect(state.snapshot).toBeNull();
    expect(state.draft).toEqual({
      doctrineFragments: 0,
      unlockedDoctrines: [],
      discoveredPlanets: ['solstice-anchor'],
      activePlanet: 'solstice-anchor',
      surveyProgress: 0,
    });
  });

  it('initializes from a real snapshot', () => {
    const snapshot = createSnapshot();
    const state = createProgressionPanelState(snapshot, {
      applyProgression: vi.fn(),
    });

    expect(state.snapshot?.run).toEqual(snapshot.run);
    expect(state.draft).toEqual({
      doctrineFragments: snapshot.run.doctrineFragments,
      unlockedDoctrines: snapshot.run.doctrineIds,
      discoveredPlanets: snapshot.run.discoveredPlanetIds,
      activePlanet: snapshot.run.activePlanetId,
      surveyProgress: 1,
    });
    expect(state.activePlanetOptions).toEqual(snapshot.run.discoveredPlanetIds);
  });

  it('applies successful updates', async () => {
    const initialSnapshot = createSnapshot();
    const nextSnapshot = createUpdatedSnapshot();
    const applyProgression = vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot });
    const state = createProgressionPanelState(initialSnapshot, {
      applyProgression,
    });

    state.setDoctrineFragments(nextSnapshot.run.doctrineFragments);
    state.toggleUnlockedDoctrine('deep-survey-protocols', true);
    state.toggleUnlockedDoctrine('hardened-relays', false);
    state.toggleUnlockedDoctrine('frontier-charters', true);
    state.toggleDiscoveredPlanet('aurora-pier', true);
    state.setActivePlanet('aurora-pier');
    state.setSurveyProgress(0.9);

    await state.apply();

    expect(applyProgression).toHaveBeenCalledWith({
      doctrineFragments: nextSnapshot.run.doctrineFragments,
      unlockedDoctrines: nextSnapshot.run.doctrineIds,
      discoveredPlanets: nextSnapshot.run.discoveredPlanetIds,
      activePlanet: nextSnapshot.run.activePlanetId,
      surveyProgress: {
        'aurora-pier': 0.9,
      },
    });
    expect(state.snapshot?.run).toEqual(nextSnapshot.run);
    expect(state.draft).toEqual({
      doctrineFragments: nextSnapshot.run.doctrineFragments,
      unlockedDoctrines: nextSnapshot.run.doctrineIds,
      discoveredPlanets: nextSnapshot.run.discoveredPlanetIds,
      activePlanet: nextSnapshot.run.activePlanetId,
      surveyProgress: 0.9,
    });
    expect(state.errorMessage).toBeNull();
  });

  it('stores failure reason codes', async () => {
    const snapshot = createSnapshot();
    const state = createProgressionPanelState(snapshot, {
      applyProgression: vi.fn().mockResolvedValue({
        ok: false,
        reasonCode: 'invalid_state',
        snapshot,
      }),
    });

    await state.apply();

    expect(state.errorMessage).toBe('invalid_state');
  });

  it('polling does not overwrite typed doctrineFragments draft', () => {
    const initialSnapshot = createSnapshot();
    const state = createProgressionPanelState(initialSnapshot, {
      applyProgression: vi.fn(),
    });

    state.setDoctrineFragments(42);
    expect(state.draft.doctrineFragments).toBe(42);

    state.sync(createUpdatedSnapshot());

    expect(state.draft.doctrineFragments).toBe(42);
  });

  it('polling does not overwrite typed surveyProgress draft', () => {
    const initialSnapshot = createSnapshot();
    const state = createProgressionPanelState(initialSnapshot, {
      applyProgression: vi.fn(),
    });

    state.setSurveyProgress(0.42);
    expect(state.draft.surveyProgress).toBe(0.42);

    state.sync(createUpdatedSnapshot());

    expect(state.draft.surveyProgress).toBe(0.42);
  });

  it('checkbox toggles for unlockedDoctrines persist across polling', () => {
    const initialSnapshot = createSnapshot();
    const state = createProgressionPanelState(initialSnapshot, {
      applyProgression: vi.fn(),
    });

    state.toggleUnlockedDoctrine('hardened-relays', false);
    state.toggleUnlockedDoctrine('deep-survey-protocols', true);

    const expectedDoctrines: DoctrineId[] = ['efficient-shifts', 'deep-survey-protocols'];
    expect(state.draft.unlockedDoctrines).toEqual(expectedDoctrines);

    state.sync(createSnapshot());

    expect(state.draft.unlockedDoctrines).toEqual(expectedDoctrines);
  });

  it('apply success in-place mutates draft fields without replacing the draft reference', async () => {
    const initialSnapshot = createSnapshot();
    const nextSnapshot = createUpdatedSnapshot();
    const applyProgression = vi.fn().mockResolvedValue({ ok: true, snapshot: nextSnapshot });
    const state = createProgressionPanelState(initialSnapshot, {
      applyProgression,
    });

    const draftRefBefore = state.draft;

    state.toggleUnlockedDoctrine('deep-survey-protocols', true);
    state.toggleUnlockedDoctrine('hardened-relays', false);
    state.toggleUnlockedDoctrine('frontier-charters', true);
    state.toggleDiscoveredPlanet('aurora-pier', true);
    state.setActivePlanet('aurora-pier');
    state.setDoctrineFragments(nextSnapshot.run.doctrineFragments);
    state.setSurveyProgress(0.9);

    await state.apply();

    expect(state.draft).toBe(draftRefBefore);

    expect(state.draft.doctrineFragments).toBe(nextSnapshot.run.doctrineFragments);
    expect(state.draft.unlockedDoctrines).toEqual(nextSnapshot.run.doctrineIds);
    expect(state.draft.discoveredPlanets).toEqual(nextSnapshot.run.discoveredPlanetIds);
    expect(state.draft.activePlanet).toBe(nextSnapshot.run.activePlanetId);
    expect(state.draft.surveyProgress).toBeCloseTo(1260 / 1400);
  });

  it('uses canonical PLANET_IDS from types', () => {
    expect(planetIds).toEqual(PLANET_IDS);
  });

  it('validates all canonical planet IDs for discovery', () => {
    const snapshot = createSnapshot();
    const state = createProgressionPanelState(snapshot, {
      applyProgression: vi.fn(),
    });

    PLANET_IDS.forEach((planetId) => {
      expect(() => {
        state.toggleDiscoveredPlanet(planetId, true);
      }).not.toThrow();

      expect(state.draft.discoveredPlanets).toContain(planetId);
    });
  });
});
