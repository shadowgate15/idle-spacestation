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

describe('Services route', () => {
  beforeEach(() => {
    clearFixture();
    vi.restoreAllMocks();
  });

  it('renders all 6 services for starter fixture', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.services.services).toHaveLength(6);

    const serviceIds = snapshot.routes.services.services.map((s) => s.id);
    expect(serviceIds).toContain('solar-harvester');
    expect(serviceIds).toContain('ore-reclaimer');
    expect(serviceIds).toContain('survey-uplink');
    expect(serviceIds).toContain('maintenance-bay');
    expect(serviceIds).toContain('command-relay');
    expect(serviceIds).toContain('fabrication-loop');
  });

  it('shows utilization summary', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.services.utilization).toEqual({
      active: 1,
      capacity: 2,
      available: 1,
      summary: '1 of 2 active service slots in use',
    });
  });

  it('shows correct service families', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.family).toBe('production');

    const maintenanceBay = snapshot.routes.services.services.find(
      (s) => s.id === 'maintenance-bay',
    );
    expect(maintenanceBay?.family).toBe('support');

    const commandRelay = snapshot.routes.services.services.find((s) => s.id === 'command-relay');
    expect(commandRelay?.family).toBe('support');

    const fabricationLoop = snapshot.routes.services.services.find(
      (s) => s.id === 'fabrication-loop',
    );
    expect(fabricationLoop?.family).toBe('conversion');
  });

  it('shows correct status for solar-harvester', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.status).toBe('active');
    expect(solarHarvester?.statusLabel).toBe('Active');
    expect(solarHarvester?.desiredActive).toBe(true);
  });

  it('shows correct status for disabled services', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const oreReclaimer = snapshot.routes.services.services.find((s) => s.id === 'ore-reclaimer');
    expect(oreReclaimer?.status).toBe('disabled');
    expect(oreReclaimer?.statusLabel).toBe('Disabled');
    expect(oreReclaimer?.desiredActive).toBe(false);
  });

  it('shows crew assignment correctly', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.crewAssignment.current).toBe(2);
    expect(solarHarvester?.crewAssignment.required).toBe(2);

    const oreReclaimer = snapshot.routes.services.services.find((s) => s.id === 'ore-reclaimer');
    expect(oreReclaimer?.crewAssignment.current).toBe(0);
    expect(oreReclaimer?.crewAssignment.required).toBe(1);
  });

  it('shows power usage correctly', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.powerUsage.upkeep).toBe(0);
    expect(solarHarvester?.powerUsage.output).toBe(4);

    const oreReclaimer = snapshot.routes.services.services.find((s) => s.id === 'ore-reclaimer');
    expect(oreReclaimer?.powerUsage.upkeep).toBe(3);
    expect(oreReclaimer?.powerUsage.output).toBe(0);

    const surveyUplink = snapshot.routes.services.services.find((s) => s.id === 'survey-uplink');
    expect(surveyUplink?.powerUsage.upkeep).toBe(2);
    expect(surveyUplink?.powerUsage.output).toBe(0);
  });

  it('shows priority order sorted by priority', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const services = snapshot.routes.services.services;
    expect(services[0]?.priorityOrder).toBe(1);
    expect(services[0]?.id).toBe('solar-harvester');
    expect(services[1]?.priorityOrder).toBe(2);
    expect(services[1]?.id).toBe('ore-reclaimer');
    expect(services[2]?.priorityOrder).toBe(3);
  });

  it('shows deficit warnings when applicable', async () => {
    const transport = setupFixtureTransport('deficit');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.services.deficitWarnings.length).toBeGreaterThan(0);

    const powerDeficit = snapshot.routes.services.deficitWarnings.find(
      (w) => w.code === 'power-deficit',
    );
    expect(powerDeficit).toBeDefined();
    expect(powerDeficit?.severity).toBe('critical');
  });

  it('shows paused status when service is paused', async () => {
    const transport = setupFixtureTransport('deficit');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const surveyUplink = snapshot.routes.services.services.find((s) => s.id === 'survey-uplink');
    expect(surveyUplink?.status).toBe('paused');
    expect(surveyUplink?.statusLabel).toBe('Paused');
  });

  it('shows disabled reasons for paused services', async () => {
    const transport = setupFixtureTransport('deficit');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const surveyUplink = snapshot.routes.services.services.find((s) => s.id === 'survey-uplink');
    expect(surveyUplink?.disabledReasons).toContain('deficit');
  });

  it('can activate service when capacity available', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    const result = await gateway.setServiceActivation({ serviceId: 'ore-reclaimer', active: true });

    expect(result.ok).toBe(true);
    if (result.ok) {
      const oreReclaimer = result.snapshot.routes.services.services.find(
        (s) => s.id === 'ore-reclaimer',
      );
      expect(oreReclaimer?.desiredActive).toBe(true);
    }
  });

  it('can pause active service', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    const result = await gateway.setServiceActivation({
      serviceId: 'solar-harvester',
      active: false,
    });

    expect(result.ok).toBe(true);
    if (result.ok) {
      const solarHarvester = result.snapshot.routes.services.services.find(
        (s) => s.id === 'solar-harvester',
      );
      expect(solarHarvester?.desiredActive).toBe(false);
    }
  });

  it('can reprioritize service up', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    const result = await gateway.reprioritizeService({
      serviceId: 'ore-reclaimer',
      direction: 'up',
    });

    expect(result.ok).toBe(true);
    if (result.ok) {
      const services = result.snapshot.routes.services.services;
      const oreReclaimer = services.find((s) => s.id === 'ore-reclaimer');
      expect(oreReclaimer?.priorityOrder).toBe(1);
    }
  });

  it('can reprioritize service down', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);

    const result = await gateway.reprioritizeService({
      serviceId: 'solar-harvester',
      direction: 'down',
    });

    expect(result.ok).toBe(true);
    if (result.ok) {
      const services = result.snapshot.routes.services.services;
      const solarHarvester = services.find((s) => s.id === 'solar-harvester');
      expect(solarHarvester?.priorityOrder).toBe(2);
    }
  });

  it('includes service description', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.description).toBeDefined();
    expect(typeof solarHarvester?.description).toBe('string');
  });

  it('shows service names formatted correctly', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const solarHarvester = snapshot.routes.services.services.find(
      (s) => s.id === 'solar-harvester',
    );
    expect(solarHarvester?.name).toBe('Solar Harvester');

    const oreReclaimer = snapshot.routes.services.services.find((s) => s.id === 'ore-reclaimer');
    expect(oreReclaimer?.name).toBe('Ore Reclaimer');
  });

  it('shows active services can exceed capacity (deficit handling)', async () => {
    const transport = setupFixtureTransport('all-planets');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.services.utilization.active).toBe(5);
    expect(snapshot.routes.services.utilization.capacity).toBe(4);
  });
});
