import { describe, expect, it, vi, beforeEach } from 'vitest';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createGameGateway } from '$lib/game/api/gateway';
import type { PreviewFixtureName } from '$lib/game/api/types';

const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';

function setupFixtureTransport(fixtureName: PreviewFixtureName) {
  const transport = createFixtureTransport(fixtureName);
  localStorage.setItem(FIXTURE_STORAGE_KEY, fixtureName);
  return transport;
}

function clearFixture() {
  localStorage.removeItem(FIXTURE_STORAGE_KEY);
}

describe('Planets route', () => {
  beforeEach(() => {
    clearFixture();
    vi.restoreAllMocks();
  });

  it('renders starter fixture with only solstice-anchor discovered', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.planets.activePlanetId).toBe('solstice-anchor');
    expect(snapshot.routes.planets.planets.length).toBe(3);

    const solsticeAnchor = snapshot.routes.planets.planets.find((p) => p.id === 'solstice-anchor');
    expect(solsticeAnchor?.discovered).toBe(true);
    expect(solsticeAnchor?.active).toBe(true);

    const cinderForge = snapshot.routes.planets.planets.find((p) => p.id === 'cinder-forge');
    expect(cinderForge?.discovered).toBe(false);
    expect(cinderForge?.selectableForNextRun).toBe(false);

    const auroraPier = snapshot.routes.planets.planets.find((p) => p.id === 'aurora-pier');
    expect(auroraPier?.discovered).toBe(false);
  });

  it('renders all-planets fixture with multiple discovered planets', async () => {
    const transport = setupFixtureTransport('all-planets');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.planets.activePlanetId).toBe('cinder-forge');

    const solsticeAnchor = snapshot.routes.planets.planets.find((p) => p.id === 'solstice-anchor');
    expect(solsticeAnchor?.discovered).toBe(true);
    expect(solsticeAnchor?.active).toBe(false);
    expect(solsticeAnchor?.selectableForNextRun).toBe(true);

    const cinderForge = snapshot.routes.planets.planets.find((p) => p.id === 'cinder-forge');
    expect(cinderForge?.discovered).toBe(true);
    expect(cinderForge?.active).toBe(true);

    const auroraPier = snapshot.routes.planets.planets.find((p) => p.id === 'aurora-pier');
    expect(auroraPier?.discovered).toBe(true);
  });

  it('includes survey progress data', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.planets.surveyProgress).toBeDefined();
    expect(snapshot.routes.planets.surveyProgress.current).toBe(0);
    expect(snapshot.routes.planets.surveyProgress.nextPlanetId).toBe('cinder-forge');
    expect(snapshot.routes.planets.surveyProgress.nextThreshold).toBe(600);
  });

  it('shows planet modifiers for discovered planets', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solsticeAnchor = snapshot.routes.planets.planets.find((p) => p.id === 'solstice-anchor');
    expect(solsticeAnchor?.modifiers.length).toBe(2);
    expect(solsticeAnchor?.modifiers.some((m) => m.target === 'crew-efficiency')).toBe(true);
    expect(solsticeAnchor?.modifiers.some((m) => m.target === 'data-output')).toBe(true);
  });

  it('shows survey thresholds for undiscovered planets', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const cinderForge = snapshot.routes.planets.planets.find((p) => p.id === 'cinder-forge');
    expect(cinderForge?.surveyThreshold).toBe(600);
    expect(cinderForge?.discovered).toBe(false);

    const auroraPier = snapshot.routes.planets.planets.find((p) => p.id === 'aurora-pier');
    expect(auroraPier?.surveyThreshold).toBe(1400);
  });

  it('shows selectability reasons for non-discovered planets', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const cinderForge = snapshot.routes.planets.planets.find((p) => p.id === 'cinder-forge');
    expect(cinderForge?.selectabilityReason).toBe(
      'Survey progress has not reached this world yet.',
    );
  });
});
