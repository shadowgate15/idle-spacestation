import type {
  ConfirmPrestigeInput,
  GameActionFailure,
  GameActionResponse,
  GameCommandName,
  GameCommandPayloads,
  GameCommandResponses,
  GameTransport,
  PreviewFixtureName,
  PreviewFixtureState,
  PurchaseDoctrineInput,
  RawServiceStateSnapshot,
  RawSystemStateSnapshot,
  ServiceActivationRejectionCode,
  ServiceCrewAssignmentRejectionCode,
  ServicePriorityRejectionCode,
  SurveyStartRejectionCode,
  SystemUpgradeRejectionCode,
} from '../types';
import {
  buildSnapshotFromFixtureState,
  cloneFixtureState,
  createFixtureState,
  currentSystemLevelValue,
  FIXTURE_STORAGE_KEY,
  isPreviewFixtureName,
  readPreviewFixtureName,
} from './fixtures';

const SYSTEM_UPGRADE_COSTS: Record<RawSystemStateSnapshot['id'], Array<number | null>> = {
  'reactor-core': [40, 80, 140, null],
  'habitat-ring': [35, 75, 130, null],
  'logistics-spine': [30, 70, 120, null],
  'survey-array': [50, 95, 155, null],
};

const PLANET_THRESHOLDS = {
  'cinder-forge': 600,
  'aurora-pier': 1400,
} as const;

const DOCTRINE_ORDER = [
  'efficient-shifts',
  'deep-survey-protocols',
  'hardened-relays',
  'frontier-charters',
] as const;

type GameRequestSaveResponse = GameCommandResponses['game_request_save'];
type GameRequestLoadResponse = GameCommandResponses['game_request_load'];

export interface PreviewFixtureTransport extends GameTransport {
  readonly fixtureName: PreviewFixtureName;
  readonly storageKey: typeof FIXTURE_STORAGE_KEY;
  getSnapshot(): GameCommandResponses['game_get_snapshot'];
}

export function createFixtureTransport(fixtureName: PreviewFixtureName): PreviewFixtureTransport {
  let state = createFixtureState(fixtureName);

  const snapshot = () => buildSnapshotFromFixtureState(state, fixtureName);

  return {
    fixtureName,
    storageKey: FIXTURE_STORAGE_KEY,
    async invoke<TCommand extends GameCommandName>(
      command: TCommand,
      payload: GameCommandPayloads[TCommand],
    ): Promise<GameCommandResponses[TCommand]> {
      switch (command) {
        case 'game_get_snapshot':
          return snapshot() as GameCommandResponses[TCommand];
        case 'game_upgrade_system':
          return upgradeSystem(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_upgrade_system']).systemId,
          ) as GameCommandResponses[TCommand];
        case 'game_set_service_activation':
          return setServiceActivation(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_set_service_activation']).serviceId,
            (payload as GameCommandPayloads['game_set_service_activation']).active,
          ) as GameCommandResponses[TCommand];
        case 'game_assign_service_crew':
          return assignServiceCrew(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_assign_service_crew']).serviceId,
            (payload as GameCommandPayloads['game_assign_service_crew']).assignedCrew,
          ) as GameCommandResponses[TCommand];
        case 'game_reprioritize_service':
          return reprioritizeService(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_reprioritize_service']).serviceId,
            (payload as GameCommandPayloads['game_reprioritize_service']).direction,
          ) as GameCommandResponses[TCommand];
        case 'game_start_survey':
          return startSurvey(state, fixtureName) as GameCommandResponses[TCommand];
        case 'game_purchase_doctrine':
          return purchaseDoctrine(
            state,
            fixtureName,
            payload as GameCommandPayloads['game_purchase_doctrine'],
          ) as GameCommandResponses[TCommand];
        case 'game_confirm_prestige':
          return confirmPrestige(
            state,
            fixtureName,
            payload as GameCommandPayloads['game_confirm_prestige'],
          ) as GameCommandResponses[TCommand];
        case 'game_request_save':
          return {
            ok: true,
            status: 'saved',
            snapshot: snapshot(),
          } as GameRequestSaveResponse as GameCommandResponses[TCommand];
        case 'game_request_load':
          state = createFixtureState(fixtureName);
          return {
            ok: true,
            status: 'loaded',
            snapshot: snapshot(),
          } as GameRequestLoadResponse as GameCommandResponses[TCommand];
      }

      return unreachable(command);
    },
    getSnapshot() {
      return snapshot();
    },
  };
}

export function maybeCreatePreviewFixtureTransport(
  storage: Pick<Storage, 'getItem'> | undefined = typeof window === 'undefined'
    ? undefined
    : window.localStorage,
): PreviewFixtureTransport | null {
  const fixtureName = readPreviewFixtureName(storage);
  return fixtureName ? createFixtureTransport(fixtureName) : null;
}

export function readPreviewFixtureSelection(
  storage: Pick<Storage, 'getItem'> | undefined = typeof window === 'undefined'
    ? undefined
    : window.localStorage,
): PreviewFixtureName | null {
  return readPreviewFixtureName(storage);
}

function upgradeSystem(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  systemId: RawSystemStateSnapshot['id'],
): GameActionResponse<SystemUpgradeRejectionCode> {
  const system = state.systems.find((entry) => entry.id === systemId);
  if (!system) {
    return failure(fixtureName, state, 'unknown-system');
  }

  const upgradeCost = SYSTEM_UPGRADE_COSTS[systemId][system.level - 1];
  if (upgradeCost === null) {
    return failure(fixtureName, state, 'max-level');
  }
  if (state.materials < upgradeCost) {
    return failure(fixtureName, state, 'insufficient-materials');
  }

  state.materials -= upgradeCost;
  system.level += 1;
  if (systemId === 'reactor-core') {
    state.power.generated =
      currentSystemLevelValue(state, 'reactor-core', 'powerOutput') ?? state.power.generated;
  }
  if (systemId === 'habitat-ring') {
    const nextCrewCap = currentSystemLevelValue(state, 'habitat-ring', 'crewCapacity');
    if (nextCrewCap !== null && state.crew.total < nextCrewCap) {
      state.crew.available += nextCrewCap - state.crew.total;
      state.crew.total = nextCrewCap;
    }
  }
  recalcPowerAvailable(state);
  return success(fixtureName, state);
}

function setServiceActivation(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  serviceId: RawServiceStateSnapshot['id'],
  active: boolean,
): GameActionResponse<ServiceActivationRejectionCode> {
  const service = state.services.find((entry) => entry.id === serviceId);
  if (!service) {
    return failure(fixtureName, state, 'unknown-service');
  }

  service.desiredActive = active;
  if (!active) {
    service.isActive = false;
    service.isPaused = false;
    service.pauseReason = null;
    service.assignedCrew = 0;
    recalcCrewTotals(state);
    recalcPowerAvailable(state);
    return success(fixtureName, state);
  }

  const activeCount = state.services.filter((entry) => entry.isActive).length;
  const activeServiceCap =
    currentSystemLevelValue(state, 'logistics-spine', 'activeServiceSlots') ?? 0;
  if (!service.isActive && activeCount >= activeServiceCap) {
    service.isPaused = true;
    service.pauseReason = 'capacity';
    return failure(fixtureName, state, 'capacity-reached');
  }

  const requiredCrew = requiredCrewForService(service.id);
  const additionalCrewNeeded = Math.max(requiredCrew - service.assignedCrew, 0);
  if (state.crew.available < additionalCrewNeeded) {
    service.isPaused = true;
    service.pauseReason = 'crew';
    return failure(fixtureName, state, 'insufficient-crew');
  }

  const projectedPowerReserved =
    state.power.reserved +
    servicePowerUpkeep(state, service.id) -
    (service.isActive ? servicePowerUpkeep(state, service.id) : 0);
  if (state.power.generated - projectedPowerReserved < 0) {
    service.isPaused = true;
    service.pauseReason = 'deficit';
    return failure(fixtureName, state, 'power-deficit');
  }

  state.crew.available -= additionalCrewNeeded;
  state.crew.assigned += additionalCrewNeeded;
  service.assignedCrew = requiredCrew;
  service.isActive = true;
  service.isPaused = false;
  service.pauseReason = null;
  recalcPowerAvailable(state);
  return success(fixtureName, state);
}

function assignServiceCrew(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  serviceId: RawServiceStateSnapshot['id'],
  assignedCrew: number,
): GameActionResponse<ServiceCrewAssignmentRejectionCode> {
  const service = state.services.find((entry) => entry.id === serviceId);
  if (!service) {
    return failure(fixtureName, state, 'unknown-service');
  }
  if (assignedCrew < 0) {
    return failure(fixtureName, state, 'invalid-assignment');
  }

  const delta = assignedCrew - service.assignedCrew;
  if (delta > state.crew.available) {
    return failure(fixtureName, state, 'insufficient-crew');
  }

  service.assignedCrew = assignedCrew;
  state.crew.assigned += delta;
  state.crew.available -= delta;
  recalcPowerAvailable(state);
  return success(fixtureName, state);
}

function reprioritizeService(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  serviceId: RawServiceStateSnapshot['id'],
  direction: 'up' | 'down',
): GameActionResponse<ServicePriorityRejectionCode> {
  const ordered = [...state.services].sort((left, right) => left.priority - right.priority);
  const index = ordered.findIndex((service) => service.id === serviceId);
  if (index === -1) {
    return failure(fixtureName, state, 'unknown-service');
  }

  const swapIndex = direction === 'up' ? index - 1 : index + 1;
  if (swapIndex < 0 || swapIndex >= ordered.length) {
    return failure(fixtureName, state, 'priority-limit');
  }

  const current = ordered[index];
  const target = ordered[swapIndex];
  const currentPriority = current.priority;
  current.priority = target.priority;
  target.priority = currentPriority;

  state.services = [...ordered].sort((left, right) => left.priority - right.priority);
  return success(fixtureName, state);
}

function startSurvey(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
): GameActionResponse<SurveyStartRejectionCode> {
  if (
    state.discoveredPlanetIds.includes('cinder-forge') &&
    state.discoveredPlanetIds.includes('aurora-pier')
  ) {
    return failure(fixtureName, state, 'all-planets-discovered');
  }

  const surveyUplink = state.services.find((service) => service.id === 'survey-uplink');
  if (!surveyUplink) {
    return failure(fixtureName, state, 'all-planets-discovered');
  }

  surveyUplink.desiredActive = true;
  surveyUplink.isActive = true;
  if (surveyUplink.assignedCrew === 0 && state.crew.available > 0) {
    surveyUplink.assignedCrew = 1;
    state.crew.assigned += 1;
    state.crew.available -= 1;
  }

  const surveyGain = 200;
  state.surveyProgress += surveyGain;
  if (
    state.surveyProgress >= PLANET_THRESHOLDS['cinder-forge'] &&
    !state.discoveredPlanetIds.includes('cinder-forge')
  ) {
    state.discoveredPlanetIds.push('cinder-forge');
  }
  if (
    state.surveyProgress >= PLANET_THRESHOLDS['aurora-pier'] &&
    !state.discoveredPlanetIds.includes('aurora-pier')
  ) {
    state.discoveredPlanetIds.push('aurora-pier');
  }
  state.discoveredPlanetIds.sort();
  state.data += 10;
  state.tickCount += 40;
  recalcPowerAvailable(state);
  return success(fixtureName, state);
}

function purchaseDoctrine(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: PurchaseDoctrineInput,
): GameActionResponse<'unknown-doctrine' | 'already-unlocked' | 'insufficient-fragments'> {
  if (!DOCTRINE_ORDER.includes(payload.doctrineId)) {
    return failure(fixtureName, state, 'unknown-doctrine');
  }
  if (state.doctrineIds.includes(payload.doctrineId)) {
    return failure(fixtureName, state, 'already-unlocked');
  }
  if (state.doctrineFragments < 1) {
    return failure(fixtureName, state, 'insufficient-fragments');
  }

  state.doctrineFragments -= 1;
  state.doctrineIds.push(payload.doctrineId);
  state.doctrineIds.sort();
  return success(fixtureName, state);
}

function confirmPrestige(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: ConfirmPrestigeInput,
): GameActionResponse<
  | 'station-tier-below-four'
  | 'needs-two-non-starter-planets'
  | 'unstable-net-power'
  | 'confirmation-required'
> {
  if (!payload.confirm) {
    return failure(fixtureName, state, 'confirmation-required');
  }

  const stationTier = calculateStationTier(state.systems);
  if (stationTier < 4) {
    return failure(fixtureName, state, 'station-tier-below-four');
  }
  if (state.discoveredPlanetIds.filter((planetId) => planetId !== 'solstice-anchor').length < 2) {
    return failure(fixtureName, state, 'needs-two-non-starter-planets');
  }
  if (state.stablePowerSeconds < 300) {
    return failure(fixtureName, state, 'unstable-net-power');
  }

  const retainedPlanets = [...state.discoveredPlanetIds];
  const retainedDoctrines = [...state.doctrineIds];
  const retainedFragments =
    state.doctrineFragments +
    Math.min(6, 1 + retainedPlanets.length - 1 + Math.floor(state.data / 1500));
  const fresh = createFixtureState('starter');

  state.tickCount = 0;
  state.stablePowerSeconds = 0;
  state.activePlanetId = 'solstice-anchor';
  state.discoveredPlanetIds = retainedPlanets;
  state.doctrineIds = retainedDoctrines;
  state.doctrineFragments = retainedFragments;
  state.surveyProgress = 0;
  state.materials = fresh.materials;
  state.data = fresh.data;
  state.power = { ...fresh.power };
  state.crew = { ...fresh.crew };
  state.systems = fresh.systems.map((system) => ({ ...system }));
  state.services = fresh.services.map((service) => ({ ...service }));
  return success(fixtureName, state);
}

function requiredCrewForService(serviceId: RawServiceStateSnapshot['id']): number {
  switch (serviceId) {
    case 'solar-harvester':
      return 2;
    default:
      return 1;
  }
}

function servicePowerUpkeep(
  state: PreviewFixtureState,
  serviceId: RawServiceStateSnapshot['id'],
): number {
  const baseUpkeep =
    serviceId === 'ore-reclaimer'
      ? 3
      : serviceId === 'survey-uplink'
        ? 2
        : serviceId === 'maintenance-bay' || serviceId === 'command-relay'
          ? 1
          : serviceId === 'fabrication-loop'
            ? 2
            : 0;
  const modifier = state.activePlanetId === 'cinder-forge' ? 1.2 : 1;
  return Number((baseUpkeep * modifier).toFixed(2));
}

function recalcCrewTotals(state: PreviewFixtureState): void {
  state.crew.assigned = state.services.reduce((sum, service) => sum + service.assignedCrew, 0);
  state.crew.available = Math.max(state.crew.total - state.crew.assigned, 0);
}

function recalcPowerAvailable(state: PreviewFixtureState): void {
  const reservedFromServices = state.services
    .filter((service) => service.isActive)
    .reduce((sum, service) => sum + servicePowerUpkeep(state, service.id), 0);
  state.power.reserved = Number((2 + reservedFromServices).toFixed(2));
  state.power.available = Number((state.power.generated - state.power.reserved).toFixed(2));
  if (state.power.available >= 0) {
    state.stablePowerSeconds = Math.max(state.stablePowerSeconds, 1);
  } else {
    state.stablePowerSeconds = 0;
  }
}

function success<TReason extends string>(
  fixtureName: PreviewFixtureName,
  state: PreviewFixtureState,
): GameActionResponse<TReason> {
  return {
    ok: true,
    snapshot: buildSnapshotFromFixtureState(cloneFixtureState(state), fixtureName),
  };
}

function failure<TReason extends string>(
  fixtureName: PreviewFixtureName,
  state: PreviewFixtureState,
  reasonCode: TReason,
): GameActionFailure<TReason> {
  return {
    ok: false,
    reasonCode,
    snapshot: buildSnapshotFromFixtureState(cloneFixtureState(state), fixtureName),
  };
}

function calculateStationTier(systems: RawSystemStateSnapshot[]): number {
  return Math.max(1, Math.min(4, systems.reduce((sum, system) => sum + system.level, 0) - 3));
}

function unreachable(value: never): never {
  void value;
  throw new Error('Unhandled game command');
}
