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

describe('Systems route', () => {
  beforeEach(() => {
    clearFixture();
    vi.restoreAllMocks();
  });

  it('renders all 4 systems for starter fixture at level 1', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.systems.systems).toHaveLength(4);

    const systemIds = snapshot.routes.systems.systems.map((s) => s.id);
    expect(systemIds).toContain('reactor-core');
    expect(systemIds).toContain('habitat-ring');
    expect(systemIds).toContain('logistics-spine');
    expect(systemIds).toContain('survey-array');
  });

  it('shows correct level and cap values for reactor-core', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reactorCore = snapshot.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.level).toBe(1);
    expect(reactorCore?.maxLevel).toBe(4);
    expect(reactorCore?.name).toBe('Reactor Core');
    expect(reactorCore?.capValues).toHaveLength(2);

    const powerOutput = reactorCore?.capValues.find((c) => c.key === 'power-output');
    expect(powerOutput?.value).toBe(8);
    expect(powerOutput?.label).toBe('Power output');

    const powerCap = reactorCore?.capValues.find((c) => c.key === 'service-power-cap');
    expect(powerCap?.value).toBe(8);
    expect(powerCap?.label).toBe('Service power cap');
  });

  it('shows correct cap values for habitat-ring', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const habitatRing = snapshot.routes.systems.systems.find((s) => s.id === 'habitat-ring');
    expect(habitatRing?.level).toBe(1);
    expect(habitatRing?.maxLevel).toBe(4);
    expect(habitatRing?.name).toBe('Habitat Ring');
    expect(habitatRing?.capValues).toHaveLength(2);

    const crewCap = habitatRing?.capValues.find((c) => c.key === 'crew-capacity');
    expect(crewCap?.value).toBe(6);
    expect(crewCap?.label).toBe('Crew capacity');

    const crewRecovery = habitatRing?.capValues.find((c) => c.key === 'crew-recovery');
    expect(crewRecovery?.value).toBe(1);
    expect(crewRecovery?.label).toBe('Crew recovery ceiling');
  });

  it('shows correct cap values for logistics-spine', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const logisticsSpine = snapshot.routes.systems.systems.find((s) => s.id === 'logistics-spine');
    expect(logisticsSpine?.level).toBe(1);
    expect(logisticsSpine?.maxLevel).toBe(4);
    expect(logisticsSpine?.name).toBe('Logistics Spine');
    expect(logisticsSpine?.capValues).toHaveLength(2);

    const slots = logisticsSpine?.capValues.find((c) => c.key === 'active-service-slots');
    expect(slots?.value).toBe(2);
    expect(slots?.label).toBe('Active service slots');

    const materialsCap = logisticsSpine?.capValues.find((c) => c.key === 'materials-capacity');
    expect(materialsCap?.value).toBe(250);
    expect(materialsCap?.label).toBe('Materials capacity');
  });

  it('shows correct cap values for survey-array', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const surveyArray = snapshot.routes.systems.systems.find((s) => s.id === 'survey-array');
    expect(surveyArray?.level).toBe(1);
    expect(surveyArray?.maxLevel).toBe(4);
    expect(surveyArray?.name).toBe('Survey Array');
    expect(surveyArray?.capValues).toHaveLength(2);

    const dataMult = surveyArray?.capValues.find((c) => c.key === 'data-multiplier');
    expect(dataMult?.value).toBe(1);
    expect(dataMult?.label).toBe('Data multiplier');

    const surveyMult = surveyArray?.capValues.find((c) => c.key === 'survey-multiplier');
    expect(surveyMult?.value).toBe(1);
    expect(surveyMult?.label).toBe('Survey multiplier');
  });

  it('shows upgrade cost for starter fixture', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reactorCore = snapshot.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.upgradeCostMaterials).toBe(40);
    expect(reactorCore?.canUpgrade).toBe(true);
    expect(reactorCore?.upgradeBlockedReason).toBeNull();

    const habitatRing = snapshot.routes.systems.systems.find((s) => s.id === 'habitat-ring');
    expect(habitatRing?.upgradeCostMaterials).toBe(35);

    const logisticsSpine = snapshot.routes.systems.systems.find((s) => s.id === 'logistics-spine');
    expect(logisticsSpine?.upgradeCostMaterials).toBe(30);

    const surveyArray = snapshot.routes.systems.systems.find((s) => s.id === 'survey-array');
    expect(surveyArray?.upgradeCostMaterials).toBe(50);
  });

  it('can upgrade reactor-core when affordable', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    const result = await gateway.upgradeSystem({ systemId: 'reactor-core' });

    expect(result.ok).toBe(true);
    if (result.ok) {
      const reactorCore = result.snapshot.routes.systems.systems.find(
        (s) => s.id === 'reactor-core',
      );
      expect(reactorCore?.level).toBe(2);
      expect(reactorCore?.canUpgrade).toBe(true);
      expect(reactorCore?.upgradeCostMaterials).toBe(80);
    }
  });

  it('shows upgrade blocked reason when materials insufficient', async () => {
    const transport = setupFixtureTransport('deficit');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reactorCore = snapshot.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.canUpgrade).toBe(true);
  });

  it('shows max level when at max', async () => {
    const transport = setupFixtureTransport('all-planets');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reactorCore = snapshot.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.level).toBe(3);
    expect(reactorCore?.canUpgrade).toBe(true);
    expect(reactorCore?.upgradeCostMaterials).toBe(140);

    const logisticsSpine = snapshot.routes.systems.systems.find((s) => s.id === 'logistics-spine');
    expect(logisticsSpine?.level).toBe(3);
  });

  it('shows correct cap values after upgrade', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    await gateway.upgradeSystem({ systemId: 'reactor-core' });
    const result = await gateway.getSnapshot();

    const reactorCore = result.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.level).toBe(2);

    const powerOutput = reactorCore?.capValues.find((c) => c.key === 'power-output');
    expect(powerOutput?.value).toBe(12);

    const powerCap = reactorCore?.capValues.find((c) => c.key === 'service-power-cap');
    expect(powerCap?.value).toBe(12);
  });

  it('includes system description', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const reactorCore = snapshot.routes.systems.systems.find((s) => s.id === 'reactor-core');
    expect(reactorCore?.description).toBeDefined();
    expect(typeof reactorCore?.description).toBe('string');
  });
});
