import { describe, expect, it } from 'vitest';

import {
  adaptGameSnapshot,
  adaptGameViewModels,
  createGameGateway,
  previewFixtureNames,
  type GameSnapshot,
  type GameCommandName,
  type GameCommandPayloads,
  type GameCommandResponses,
  type GameTransport,
} from './index';
import {
  buildSnapshotFromFixtureState,
  createFixtureState,
  createFixtureTransport,
  starter,
} from './testing';

describe('game api fixtures and adapters', () => {
  it('sends camelCase payloads for service activation commands', async () => {
    const calls: Array<{ command: string; payload: unknown }> = [];
    const transport: GameTransport = {
      invoke: async <TCommand extends GameCommandName>(
        command: TCommand,
        payload: GameCommandPayloads[TCommand],
      ): Promise<GameCommandResponses[TCommand]> => {
        calls.push({ command, payload });

        if (command === 'game_set_service_activation') {
          return {
            ok: true,
            snapshot: buildSnapshotFromFixtureState(createFixtureState('starter'), 'starter'),
          } as GameCommandResponses[TCommand];
        }

        throw new Error(`Unexpected command: ${command}`);
      },
    };
    const gateway = createGameGateway(transport);

    await gateway.setServiceActivation({ serviceId: 'solar-harvester', active: true });

    expect(calls).toEqual([
      {
        command: 'game_set_service_activation',
        payload: { serviceId: 'solar-harvester', active: true },
      },
    ]);
  });

  it('sends camelCase payloads for system upgrade commands', async () => {
    const calls: Array<{ command: string; payload: unknown }> = [];
    const transport: GameTransport = {
      invoke: async <TCommand extends GameCommandName>(
        command: TCommand,
        payload: GameCommandPayloads[TCommand],
      ): Promise<GameCommandResponses[TCommand]> => {
        calls.push({ command, payload });

        if (command === 'game_upgrade_system') {
          return {
            ok: true,
            snapshot: buildSnapshotFromFixtureState(createFixtureState('starter'), 'starter'),
          } as GameCommandResponses[TCommand];
        }

        throw new Error(`Unexpected command: ${command}`);
      },
    };
    const gateway = createGameGateway(transport);

    await gateway.upgradeSystem({ systemId: 'reactor-core' });

    expect(calls).toEqual([
      {
        command: 'game_upgrade_system',
        payload: { systemId: 'reactor-core' },
      },
    ]);
  });

  it.each(previewFixtureNames)(
    'returns complete route-facing view models for %s',
    async (fixtureName) => {
      const gateway = createGameGateway(createFixtureTransport(fixtureName));
      const snapshot = await gateway.getSnapshot();

      assertCompleteSnapshot(snapshot);
    },
  );

  it('adapters consume route snapshots without recomputing route-level math', () => {
    const rawSnapshot = buildSnapshotFromFixtureState(
      createFixtureState('all-planets'),
      'all-planets',
    );

    const adaptedSnapshot = adaptGameSnapshot(rawSnapshot);
    const adaptedViewModels = adaptGameViewModels(rawSnapshot);

    expect(adaptedSnapshot.routes).toEqual(rawSnapshot.routeSnapshots);
    expect(adaptedViewModels).toEqual(rawSnapshot.routeSnapshots);
    expect(adaptedSnapshot.routes).not.toBe(rawSnapshot.routeSnapshots);
    expect(adaptedSnapshot.routes.overview).not.toBe(rawSnapshot.routeSnapshots.overview);
    expect(adaptedSnapshot.routes.systems).not.toBe(rawSnapshot.routeSnapshots.systems);
    expect(adaptedSnapshot.routes.services).not.toBe(rawSnapshot.routeSnapshots.services);
    expect(adaptedSnapshot.routes.planets).not.toBe(rawSnapshot.routeSnapshots.planets);
    expect(adaptedSnapshot.routes.prestige).not.toBe(rawSnapshot.routeSnapshots.prestige);
  });

  it('matches the exact starter fresh-profile seed contract', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));
    const snapshot = await gateway.getSnapshot();

    expect(starter.activePlanetId).toBe('solstice-anchor');
    expect(snapshot.meta.fixtureName).toBe('starter');
    expect(snapshot.run.activePlanetId).toBe('solstice-anchor');
    expect(snapshot.routes.overview.activePlanet.name).toBe('Solstice Anchor');
    expect(
      snapshot.routes.overview.activePlanet.modifiers.map((modifier) => modifier.effectText),
    ).toEqual(['+10% Crew efficiency', '-10% Data output']);

    expect(snapshot.resources.materials).toBe(120);
    expect(snapshot.resources.data).toBe(0);
    expect(snapshot.resources.crew).toEqual({ total: 6, assigned: 2, available: 4 });
    expect(snapshot.resources.power).toEqual({ generated: 8, reserved: 2, available: 6 });

    expect(snapshot.systems.map((system) => system.level)).toEqual([1, 1, 1, 1]);
    expect(snapshot.routes.systems.systems.every((system) => system.level === 1)).toBe(true);

    expect(
      snapshot.services.map((service) => [service.id, service.desiredActive, service.priority]),
    ).toEqual([
      ['solar-harvester', true, 1],
      ['ore-reclaimer', false, 2],
      ['survey-uplink', false, 3],
      ['maintenance-bay', false, 4],
      ['command-relay', false, 5],
      ['fabrication-loop', false, 6],
    ]);

    const activeServices = snapshot.routes.services.services.filter(
      (service) => service.status === 'active',
    );
    expect(activeServices).toHaveLength(1);
    expect(activeServices[0]?.id).toBe('solar-harvester');
    expect(snapshot.routes.services.utilization).toEqual({
      active: 1,
      capacity: 2,
      available: 1,
      summary: '1 of 2 active service slots in use',
    });
  });
});

function assertCompleteSnapshot(snapshot: GameSnapshot): void {
  expect(snapshot.meta.source).toBe('preview-fixture');
  expect(snapshot.routes.overview.activePlanet.id).toMatch(
    /solstice-anchor|cinder-forge|aurora-pier/,
  );
  expect(snapshot.routes.overview.activePlanet.modifiers.length).toBeGreaterThan(0);
  expect(snapshot.routes.overview.resourceDeltas.length).toBeGreaterThanOrEqual(3);
  expect(snapshot.routes.overview.stationTier.current).toBeGreaterThanOrEqual(1);
  expect(snapshot.routes.overview.serviceUtilization.capacity).toBeGreaterThan(0);
  expect(typeof snapshot.routes.overview.surveyProgress.summary).toBe('string');

  expect(snapshot.routes.systems.systems).toHaveLength(4);
  expect(snapshot.routes.systems.systems.every((system) => system.capValues.length > 0)).toBe(true);

  expect(snapshot.routes.services.services).toHaveLength(6);
  expect(
    snapshot.routes.services.services.every((service) => typeof service.statusLabel === 'string'),
  ).toBe(true);

  expect(snapshot.routes.planets.planets).toHaveLength(3);
  expect(snapshot.routes.planets.planets.every((planet) => planet.modifiers.length > 0)).toBe(true);

  expect(typeof snapshot.routes.prestige.eligibility.eligible).toBe('boolean');
  expect(snapshot.routes.prestige.unlockedDoctrines).toBeDefined();
  expect(snapshot.routes.prestige.purchaseOptions).toHaveLength(4);
  expect(snapshot.routes.prestige.resetConsequences.length).toBeGreaterThan(0);
}
