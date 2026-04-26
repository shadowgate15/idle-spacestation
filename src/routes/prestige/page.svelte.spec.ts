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

describe('Prestige route', () => {
  beforeEach(() => {
    clearFixture();
    vi.restoreAllMocks();
  });

  it('renders starter fixture as ineligible', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.eligibility.eligible).toBe(false);
    expect(snapshot.routes.prestige.eligibility.reasonCodes.length).toBeGreaterThan(0);
  });

  it('renders all-planets fixture with partial eligibility', async () => {
    const transport = setupFixtureTransport('all-planets');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.eligibility.eligible).toBe(false);
    expect(snapshot.routes.prestige.eligibility.reasonCodes).toContain('unstable-net-power');
    expect(snapshot.routes.prestige.eligibility.stablePowerSeconds).toBe(96);
  });

  it('renders prestige-ready fixture as eligible', async () => {
    const transport = setupFixtureTransport('prestige-ready');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.eligibility.eligible).toBe(true);
    expect(snapshot.routes.prestige.eligibility.reasonCodes.length).toBe(0);
    expect(snapshot.routes.prestige.eligibility.stablePowerSeconds).toBe(300);
  });

  it('shows specific reason codes when ineligible', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reasons = snapshot.routes.prestige.eligibility.reasonCodes;
    expect(reasons).toContain('station-tier-below-four');
    expect(reasons).toContain('needs-two-non-starter-planets');
    expect(reasons).toContain('unstable-net-power');
  });

  it('includes doctrine fragments count', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.doctrineFragments).toBe(0);
  });

  it('shows unlocked doctrines', async () => {
    const transport = setupFixtureTransport('prestige-ready');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.unlockedDoctrines.length).toBe(2);
    expect(
      snapshot.routes.prestige.unlockedDoctrines.some((d) => d.id === 'efficient-shifts'),
    ).toBe(true);
    expect(
      snapshot.routes.prestige.unlockedDoctrines.some((d) => d.id === 'deep-survey-protocols'),
    ).toBe(true);
  });

  it('provides doctrine purchase options', async () => {
    const transport = setupFixtureTransport('prestige-ready');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.purchaseOptions.length).toBe(4);

    const efficientShifts = snapshot.routes.prestige.purchaseOptions.find(
      (o) => o.id === 'efficient-shifts',
    );
    expect(efficientShifts?.available).toBe(false); // already owned
    expect(efficientShifts?.blockedReason).toBe('already-unlocked');

    const frontierCharters = snapshot.routes.prestige.purchaseOptions.find(
      (o) => o.id === 'frontier-charters',
    );
    expect(frontierCharters?.available).toBe(true);
    expect(frontierCharters?.costFragments).toBe(1);
  });

  it('shows reset consequences table', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.resetConsequences.length).toBe(7);

    const discoveredPlanets = snapshot.routes.prestige.resetConsequences.find(
      (c) => c.label === 'Discovered planets',
    );
    expect(discoveredPlanets?.outcome).toBe('retain');

    const materialsData = snapshot.routes.prestige.resetConsequences.find(
      (c) => c.label === 'Materials and Data',
    );
    expect(materialsData?.outcome).toBe('reset');

    const systemLevels = snapshot.routes.prestige.resetConsequences.find(
      (c) => c.label === 'System levels and survey progress',
    );
    expect(systemLevels?.outcome).toBe('reset');
  });

  it('blocks doctrine purchase when fragments = 0', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.doctrineFragments).toBe(0);

    const purchaseOptions = snapshot.routes.prestige.purchaseOptions;
    expect(purchaseOptions.every((o) => !o.available)).toBe(true);
    expect(purchaseOptions.every((o) => o.blockedReason === 'insufficient-fragments')).toBe(true);
  });

  it('includes stable power time in eligibility', async () => {
    const transport = setupFixtureTransport('prestige-ready');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.prestige.eligibility.requiredStablePowerSeconds).toBe(300);
    expect(snapshot.routes.prestige.eligibility.stablePowerSeconds).toBe(300);
  });
});
