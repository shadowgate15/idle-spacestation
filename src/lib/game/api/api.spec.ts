import { describe, expect, expectTypeOf, it } from 'vitest';

import {
  adaptGameSnapshot,
  adaptGameViewModels,
  createGameGateway,
  previewFixtureNames,
  type DevtoolsCommandName,
  type DevtoolsCommandPayloads,
  type DevtoolsCommandResponses,
  type DevtoolsApplyResourcesRejectionCode,
  type GameSnapshot,
  type GameCommandName,
  type GameCommandPayloads,
  type GameCommandResponses,
  type GameGatewayTransport,
  type GameTransport,
  type RawDevtoolsStateSnapshot,
  type RawGameSnapshot,
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

  it('sends camelCase payloads for all devtools commands', async () => {
    const calls: Array<{ command: string; payload: unknown }> = [];
    const rawSnapshot = createRawSnapshot();
    const transport = createDevtoolsTransport(async (command, payload) => {
      calls.push({ command, payload });

      switch (command) {
        case 'game_devtools_get_state':
        case 'game_devtools_set_visibility':
          return createRawDevtoolsState(rawSnapshot, true);
        case 'game_devtools_apply_resources':
        case 'game_devtools_apply_crew':
        case 'game_devtools_apply_systems':
        case 'game_devtools_apply_services':
        case 'game_devtools_apply_progression':
        case 'game_devtools_advance_ticks':
        case 'game_devtools_reset_to_starter':
          return createRawActionSuccess(rawSnapshot);
      }
    });
    const gateway = createGameGateway(transport);

    await gateway.getDevtoolsState();
    await gateway.setDevtoolsVisibility({ visible: true });
    await gateway.applyResources({ materials: 240, data: 60 });
    await gateway.applyCrew({ crewTotal: 8 });
    await gateway.applySystems({
      systems: [
        { id: 'reactor-core', level: 2 },
        { id: 'survey-array', level: 3 },
      ],
    });
    await gateway.applyServices({
      services: [
        {
          id: 'solar-harvester',
          desiredActive: true,
          assignedCrew: 2,
          priority: 1,
        },
        {
          id: 'survey-uplink',
          desiredActive: false,
          assignedCrew: 0,
          priority: 2,
        },
      ],
    });
    await gateway.applyProgression({
      doctrineFragments: 2,
      unlockedDoctrines: ['efficient-shifts'],
      discoveredPlanets: ['solstice-anchor', 'cinder-forge'],
      activePlanet: 'cinder-forge',
      surveyProgress: {
        'cinder-forge': 600,
        'aurora-pier': 200,
      },
    });
    await gateway.advanceTicks({ count: 4 });
    await gateway.resetToStarter();

    expect(calls).toEqual([
      {
        command: 'game_devtools_get_state',
        payload: undefined,
      },
      {
        command: 'game_devtools_set_visibility',
        payload: { visible: true },
      },
      {
        command: 'game_devtools_apply_resources',
        payload: { materials: 240, data: 60 },
      },
      {
        command: 'game_devtools_apply_crew',
        payload: { crewTotal: 8 },
      },
      {
        command: 'game_devtools_apply_systems',
        payload: {
          systems: [
            { id: 'reactor-core', level: 2 },
            { id: 'survey-array', level: 3 },
          ],
        },
      },
      {
        command: 'game_devtools_apply_services',
        payload: {
          services: [
            {
              id: 'solar-harvester',
              desiredActive: true,
              assignedCrew: 2,
              priority: 1,
            },
            {
              id: 'survey-uplink',
              desiredActive: false,
              assignedCrew: 0,
              priority: 2,
            },
          ],
        },
      },
      {
        command: 'game_devtools_apply_progression',
        payload: {
          doctrineFragments: 2,
          unlockedDoctrines: ['efficient-shifts'],
          discoveredPlanets: ['solstice-anchor', 'cinder-forge'],
          activePlanet: 'cinder-forge',
          surveyProgress: {
            'cinder-forge': 600,
            'aurora-pier': 200,
          },
        },
      },
      {
        command: 'game_devtools_advance_ticks',
        payload: { count: 4 },
      },
      {
        command: 'game_devtools_reset_to_starter',
        payload: {},
      },
    ]);
  });

  it('adapts devtools state and action responses with typed result contracts', async () => {
    const rawSnapshot = createRawSnapshot();
    const transport = createDevtoolsTransport(async (command) => {
      switch (command) {
        case 'game_devtools_get_state':
          return createRawDevtoolsState(rawSnapshot, true);
        case 'game_devtools_set_visibility':
          return createRawDevtoolsState(rawSnapshot, false);
        case 'game_devtools_apply_resources':
          return {
            ok: false,
            reasonCode: 'invalid_range',
            snapshot: rawSnapshot,
          };
        case 'game_devtools_advance_ticks':
          return createRawActionSuccess(rawSnapshot);
        default:
          throw new Error(`Unhandled command: ${String(command)}`);
      }
    });
    const gateway = createGameGateway(transport);

    const state = await gateway.getDevtoolsState();
    const visibility = await gateway.setDevtoolsVisibility({ visible: false });
    const resources = await gateway.applyResources({ materials: -1, data: 10 });
    const ticks = await gateway.advanceTicks({ count: 8 });

    expect(state).toEqual({ visible: true, snapshot: adaptGameSnapshot(rawSnapshot) });
    expect(visibility).toEqual({ visible: false, snapshot: adaptGameSnapshot(rawSnapshot) });
    expect(resources).toEqual({
      ok: false,
      reasonCode: 'invalid_range',
      snapshot: adaptGameSnapshot(rawSnapshot),
    });
    expect(ticks).toEqual({
      ok: true,
      snapshot: adaptGameSnapshot(rawSnapshot),
    });

    expectTypeOf(state.visible).toEqualTypeOf<boolean>();
    expectTypeOf(state.snapshot).toEqualTypeOf<GameSnapshot>();

    if (resources.ok) {
      throw new Error('Expected applyResources to fail');
    }

    expect(resources.reasonCode).toBe('invalid_range');
    expectTypeOf(resources.reasonCode).toEqualTypeOf<DevtoolsApplyResourcesRejectionCode>();
  });

  it('does not mutate devtools state before an explicit apply call', async () => {
    let rawSnapshot = createRawSnapshot();
    const transport = createDevtoolsTransport(async (command, payload) => {
      switch (command) {
        case 'game_devtools_get_state':
          return createRawDevtoolsState(rawSnapshot, false);
        case 'game_devtools_apply_resources': {
          const resourcesPayload = payload as DevtoolsCommandPayloads['game_devtools_apply_resources'];
          rawSnapshot = {
            ...structuredClone(rawSnapshot),
            resources: {
              ...structuredClone(rawSnapshot.resources),
              materials: resourcesPayload.materials,
              data: resourcesPayload.data,
            },
          };
          return createRawActionSuccess(rawSnapshot);
        }
        default:
          throw new Error(`Unhandled command: ${String(command)}`);
      }
    });
    const gateway = createGameGateway(transport);

    const initialState = await gateway.getDevtoolsState();
    const draft = { materials: 999, data: 321 };
    const unchangedState = await gateway.getDevtoolsState();

    expect(initialState.snapshot.resources.materials).toBe(120);
    expect(initialState.snapshot.resources.data).toBe(0);
    expect(unchangedState.snapshot.resources.materials).toBe(120);
    expect(unchangedState.snapshot.resources.data).toBe(0);

    const applied = await gateway.applyResources(draft);

    expect(applied.ok).toBe(true);
    if (!applied.ok) {
      throw new Error('Expected applyResources to succeed');
    }

    expect(applied.snapshot.resources.materials).toBe(999);
    expect(applied.snapshot.resources.data).toBe(321);
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

describe('fixture transport devtools commands', () => {
  it('game_devtools_get_state returns current visibility false initially', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const state = await gateway.getDevtoolsState();

    expect(state.visible).toBe(false);
  });

  it('game_devtools_set_visibility toggles visibility and returns updated state', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.setDevtoolsVisibility({ visible: true });

    expect(result.visible).toBe(true);
    expect(result.snapshot).toBeDefined();
  });

  it('game_devtools_apply_resources updates materials and data', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.applyResources({ materials: 500, data: 100 });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      throw new Error('Expected applyResources to succeed');
    }
    expect(result.snapshot.resources.materials).toBe(500);
    expect(result.snapshot.resources.data).toBe(100);
  });

  it('game_devtools_apply_resources rejects negative values', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));
    const before = await gateway.getDevtoolsState();

    const result = await gateway.applyResources({ materials: -1, data: 10 });

    expect(result).toMatchObject({ ok: false, reasonCode: 'invalid_range' });
    expect(result.snapshot).toEqual(before.snapshot);
  });

  it('game_devtools_apply_crew updates crew total', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.applyCrew({ crewTotal: 4 });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      throw new Error('Expected applyCrew to succeed');
    }
    expect(result.snapshot.resources.crew).toEqual({ total: 4, assigned: 2, available: 2 });
  });

  it('game_devtools_apply_crew rejects below assigned count', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.applyCrew({ crewTotal: 1 });

    expect(result).toMatchObject({ ok: false, reasonCode: 'invalid_range' });
  });

  it('game_devtools_apply_systems sets system levels and recomputes power', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.applySystems({
      systems: [{ id: 'reactor-core', level: 2 }],
    });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      throw new Error('Expected applySystems to succeed');
    }
    expect(result.snapshot.resources.power.generated).toBe(12);
  });

  it('game_devtools_apply_systems rejects unknown system id', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.applySystems({
      systems: [{ id: 'unknown-system' as never, level: 2 }],
    });

    expect(result).toMatchObject({ ok: false, reasonCode: 'unknown_id' });
  });

  it('game_devtools_advance_ticks advances tick count', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.advanceTicks({ count: 10 });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      throw new Error('Expected advanceTicks to succeed');
    }
    expect(result.snapshot.meta.tickCount).toBe(10);
  });

  it('game_devtools_advance_ticks rejects count over 240', async () => {
    const gateway = createGameGateway(createFixtureTransport('starter'));

    const result = await gateway.advanceTicks({ count: 241 });

    expect(result).toMatchObject({ ok: false, reasonCode: 'invalid_range' });
  });

  it('game_devtools_reset_to_starter resets all state', async () => {
    const gateway = createGameGateway(createFixtureTransport('all-planets'));

    const result = await gateway.resetToStarter();

    expect(result.ok).toBe(true);
    if (!result.ok) {
      throw new Error('Expected resetToStarter to succeed');
    }
    expect(result.snapshot.resources.materials).toBe(120);
    expect(result.snapshot.resources.data).toBe(0);
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

function createRawSnapshot(): RawGameSnapshot {
  return buildSnapshotFromFixtureState(createFixtureState('starter'), 'starter');
}

function createRawDevtoolsState(
  snapshot: RawGameSnapshot,
  visible: boolean,
): RawDevtoolsStateSnapshot {
  return {
    visible,
    snapshot,
  };
}

function createRawActionSuccess(snapshot: RawGameSnapshot) {
  return {
    ok: true as const,
    snapshot,
  };
}

function createDevtoolsTransport(
  handler: (
    command: DevtoolsCommandName,
    payload: DevtoolsCommandPayloads[DevtoolsCommandName],
  ) => Promise<DevtoolsCommandResponses[DevtoolsCommandName]>,
): GameGatewayTransport {
  return {
    invoke(command, payload) {
      return handler(
        command as DevtoolsCommandName,
        payload as DevtoolsCommandPayloads[DevtoolsCommandName],
      ) as Promise<DevtoolsCommandResponses[DevtoolsCommandName]>;
    },
  } as GameGatewayTransport;
}
