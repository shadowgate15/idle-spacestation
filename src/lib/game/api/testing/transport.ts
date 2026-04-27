import type {
  ConfirmPrestigeInput,
  DevtoolsAdvanceTicksPayload,
  DevtoolsAdvanceTicksRejectionCode,
  DevtoolsApplyCrewPayload,
  DevtoolsApplyCrewRejectionCode,
  DevtoolsApplyProgressionPayload,
  DevtoolsApplyProgressionRejectionCode,
  DevtoolsApplyResourcesPayload,
  DevtoolsApplyResourcesRejectionCode,
  DevtoolsApplyServicesPayload,
  DevtoolsApplyServicesRejectionCode,
  DevtoolsApplySystemsPayload,
  DevtoolsApplySystemsRejectionCode,
  DevtoolsCommandName,
  DevtoolsCommandPayloads,
  DevtoolsCommandResponses,
  DevtoolsCommandTransport,
  DevtoolsResetToStarterPayload,
  GameActionFailure,
  GameActionResponse,
  GameCommandName,
  GameCommandPayloads,
  GameCommandResponses,
  GameTransport,
  PreviewFixtureName,
  PreviewFixtureState,
  PurchaseDoctrineInput,
  RawDevtoolsGetStateResponse,
  RawDevtoolsSetVisibilityResponse,
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

const DEVTOOLS_SYSTEM_IDS = [
  'reactor-core',
  'habitat-ring',
  'logistics-spine',
  'survey-array',
] as const;

const DEVTOOLS_SERVICE_IDS = [
  'solar-harvester',
  'ore-reclaimer',
  'survey-uplink',
  'maintenance-bay',
  'command-relay',
  'fabrication-loop',
] as const;

const DEVTOOLS_PLANET_IDS = ['solstice-anchor', 'cinder-forge', 'aurora-pier'] as const;

type FixtureCommandName = GameCommandName | DevtoolsCommandName;
type FixtureCommandPayloads = GameCommandPayloads & DevtoolsCommandPayloads;
type FixtureCommandResponses = GameCommandResponses & DevtoolsCommandResponses;

type GameRequestSaveResponse = GameCommandResponses['game_request_save'];
type GameRequestLoadResponse = GameCommandResponses['game_request_load'];

export type PreviewFixtureTransport = GameTransport &
  DevtoolsCommandTransport & {
    readonly fixtureName: PreviewFixtureName;
    readonly storageKey: typeof FIXTURE_STORAGE_KEY;
    getSnapshot(): GameCommandResponses['game_get_snapshot'];
  };

export function createFixtureTransport(fixtureName: PreviewFixtureName): PreviewFixtureTransport {
  let state = createFixtureState(fixtureName);
  let devtoolsVisible = false;

  const snapshot = () => buildSnapshotFromFixtureState(state, fixtureName);

  return {
    fixtureName,
    storageKey: FIXTURE_STORAGE_KEY,
    async invoke<TCommand extends FixtureCommandName>(
      command: TCommand,
      payload: FixtureCommandPayloads[TCommand],
    ): Promise<FixtureCommandResponses[TCommand]> {
      switch (command) {
        case 'game_get_snapshot':
          return snapshot() as FixtureCommandResponses[TCommand];
        case 'game_upgrade_system':
          return upgradeSystem(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_upgrade_system']).systemId,
          ) as FixtureCommandResponses[TCommand];
        case 'game_set_service_activation':
          return setServiceActivation(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_set_service_activation']).serviceId,
            (payload as GameCommandPayloads['game_set_service_activation']).active,
          ) as FixtureCommandResponses[TCommand];
        case 'game_assign_service_crew':
          return assignServiceCrew(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_assign_service_crew']).serviceId,
            (payload as GameCommandPayloads['game_assign_service_crew']).assignedCrew,
          ) as FixtureCommandResponses[TCommand];
        case 'game_reprioritize_service':
          return reprioritizeService(
            state,
            fixtureName,
            (payload as GameCommandPayloads['game_reprioritize_service']).serviceId,
            (payload as GameCommandPayloads['game_reprioritize_service']).direction,
          ) as FixtureCommandResponses[TCommand];
        case 'game_start_survey':
          return startSurvey(state, fixtureName) as FixtureCommandResponses[TCommand];
        case 'game_purchase_doctrine':
          return purchaseDoctrine(
            state,
            fixtureName,
            payload as GameCommandPayloads['game_purchase_doctrine'],
          ) as FixtureCommandResponses[TCommand];
        case 'game_confirm_prestige':
          return confirmPrestige(
            state,
            fixtureName,
            payload as GameCommandPayloads['game_confirm_prestige'],
          ) as FixtureCommandResponses[TCommand];
        case 'game_request_save':
          return {
            ok: true,
            status: 'saved',
            snapshot: snapshot(),
          } as GameRequestSaveResponse as FixtureCommandResponses[TCommand];
        case 'game_request_load':
          state = createFixtureState(fixtureName);
          return {
            ok: true,
            status: 'loaded',
            snapshot: snapshot(),
          } as GameRequestLoadResponse as FixtureCommandResponses[TCommand];
        case 'game_devtools_get_state':
          return buildDevtoolsStateResponse(
            snapshot(),
            devtoolsVisible,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_set_visibility':
          devtoolsVisible =
            (payload as DevtoolsCommandPayloads['game_devtools_set_visibility']).visible;
          return buildDevtoolsVisibilityResponse(
            snapshot(),
            devtoolsVisible,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_apply_resources':
          return applyResources(
            state,
            fixtureName,
            payload as DevtoolsApplyResourcesPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_apply_crew':
          return applyCrew(
            state,
            fixtureName,
            payload as DevtoolsApplyCrewPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_apply_systems':
          return applySystems(
            state,
            fixtureName,
            payload as DevtoolsApplySystemsPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_apply_services':
          return applyServices(
            state,
            fixtureName,
            payload as DevtoolsApplyServicesPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_apply_progression':
          return applyProgression(
            state,
            fixtureName,
            payload as DevtoolsApplyProgressionPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_advance_ticks':
          return advanceTicks(
            state,
            fixtureName,
            payload as DevtoolsAdvanceTicksPayload,
          ) as FixtureCommandResponses[TCommand];
        case 'game_devtools_reset_to_starter':
          return resetToStarter(
            state,
            fixtureName,
            payload as DevtoolsResetToStarterPayload,
          ) as FixtureCommandResponses[TCommand];
      }

      return unreachable(command as never);
    },
    getSnapshot() {
      return snapshot();
    },
  } as PreviewFixtureTransport;
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

function applyResources(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsApplyResourcesPayload,
): GameActionResponse<DevtoolsApplyResourcesRejectionCode> {
  if (!isNonNegativeNumber(payload.materials) || !isNonNegativeNumber(payload.data)) {
    return failure(fixtureName, state, 'invalid_range');
  }

  const next = cloneFixtureState(state);
  next.materials = payload.materials;
  next.data = payload.data;
  recalcPowerAvailable(next);

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function applyCrew(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsApplyCrewPayload,
): GameActionResponse<DevtoolsApplyCrewRejectionCode> {
  if (!Number.isInteger(payload.crewTotal) || payload.crewTotal < 1) {
    return failure(fixtureName, state, 'invalid_range');
  }
  if (payload.crewTotal < state.crew.assigned) {
    return failure(fixtureName, state, 'invalid_range');
  }

  const crewCapacity = currentSystemLevelValue(state, 'habitat-ring', 'crewCapacity');
  if (crewCapacity !== null && payload.crewTotal > crewCapacity) {
    return failure(fixtureName, state, 'constraint_violation');
  }

  const next = cloneFixtureState(state);
  next.crew.total = payload.crewTotal;
  recalcCrewTotals(next);
  recalcPowerAvailable(next);

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function applySystems(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsApplySystemsPayload,
): GameActionResponse<DevtoolsApplySystemsRejectionCode> {
  for (const entry of payload.systems) {
    if (!DEVTOOLS_SYSTEM_IDS.includes(entry.id)) {
      return failure(fixtureName, state, 'unknown_id');
    }
    if (!Number.isInteger(entry.level) || entry.level < 1 || entry.level > 4) {
      return failure(fixtureName, state, 'invalid_range');
    }
  }

  const next = cloneFixtureState(state);
  const reactorChanged = payload.systems.some((entry) => entry.id === 'reactor-core');
  const habitatChanged = payload.systems.some((entry) => entry.id === 'habitat-ring');

  for (const entry of payload.systems) {
    const system = next.systems.find((candidate) => candidate.id === entry.id);
    if (!system) {
      return failure(fixtureName, state, 'unknown_id');
    }
    system.level = entry.level;
  }

  if (reactorChanged) {
    next.power.generated =
      currentSystemLevelValue(next, 'reactor-core', 'powerOutput') ?? next.power.generated;
  }

  if (habitatChanged) {
    const nextCrewCapacity = currentSystemLevelValue(next, 'habitat-ring', 'crewCapacity');
    if (nextCrewCapacity !== null && nextCrewCapacity > next.crew.total) {
      const crewDelta = nextCrewCapacity - next.crew.total;
      next.crew.total = nextCrewCapacity;
      next.crew.available += crewDelta;
    }
  }

  recalcCrewTotals(next);
  recalcPowerAvailable(next);

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function applyServices(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsApplyServicesPayload,
): GameActionResponse<DevtoolsApplyServicesRejectionCode> {
  const serviceCount = state.services.length;
  const seenIds = new Set<string>();

  for (const entry of payload.services) {
    if (!state.services.some((service) => service.id === entry.id)) {
      return failure(fixtureName, state, 'unknown_id');
    }
    if (seenIds.has(entry.id)) {
      return failure(fixtureName, state, 'constraint_violation');
    }
    seenIds.add(entry.id);
    if (!Number.isInteger(entry.assignedCrew) || entry.assignedCrew < 0) {
      return failure(fixtureName, state, 'invalid_range');
    }
    if (!Number.isInteger(entry.priority) || entry.priority < 1 || entry.priority > serviceCount) {
      return failure(fixtureName, state, 'invalid_range');
    }
  }

  const next = cloneFixtureState(state);

  for (const entry of payload.services) {
    const service = next.services.find((candidate) => candidate.id === entry.id);
    if (!service) {
      return failure(fixtureName, state, 'unknown_id');
    }

    service.desiredActive = entry.desiredActive;
    service.assignedCrew = entry.assignedCrew;
    service.priority = entry.priority;
  }

  next.services.sort((left, right) => left.priority - right.priority);
  if (new Set(next.services.map((service) => service.priority)).size !== next.services.length) {
    return failure(fixtureName, state, 'constraint_violation');
  }

  deriveServiceRuntimeState(next);
  recalcCrewTotals(next);
  recalcPowerAvailable(next);

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function applyProgression(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsApplyProgressionPayload,
): GameActionResponse<DevtoolsApplyProgressionRejectionCode> {
  if (!Number.isInteger(payload.doctrineFragments) || payload.doctrineFragments < 0) {
    return failure(fixtureName, state, 'invalid_range');
  }

  for (const doctrineId of payload.unlockedDoctrines) {
    if (!DOCTRINE_ORDER.includes(doctrineId)) {
      return failure(fixtureName, state, 'unknown_id');
    }
  }

  for (const planetId of payload.discoveredPlanets) {
    if (!DEVTOOLS_PLANET_IDS.includes(planetId)) {
      return failure(fixtureName, state, 'unknown_id');
    }
  }

  if (!DEVTOOLS_PLANET_IDS.includes(payload.activePlanet)) {
    return failure(fixtureName, state, 'unknown_id');
  }

  if (!payload.discoveredPlanets.includes(payload.activePlanet)) {
    return failure(fixtureName, state, 'constraint_violation');
  }

  if (!payload.discoveredPlanets.includes('solstice-anchor')) {
    return failure(fixtureName, state, 'constraint_violation');
  }

  for (const [planetId, progress] of Object.entries(payload.surveyProgress)) {
    if (!DEVTOOLS_PLANET_IDS.includes(planetId as (typeof DEVTOOLS_PLANET_IDS)[number])) {
      return failure(fixtureName, state, 'unknown_id');
    }
    if (!isNonNegativeNumber(progress)) {
      return failure(fixtureName, state, 'invalid_range');
    }
  }

  const next = cloneFixtureState(state);
  next.doctrineFragments = payload.doctrineFragments;
  next.doctrineIds = sortByOrder(payload.unlockedDoctrines, DOCTRINE_ORDER);
  next.discoveredPlanetIds = sortByOrder(payload.discoveredPlanets, DEVTOOLS_PLANET_IDS);
  next.activePlanetId = payload.activePlanet;
  next.surveyProgress = calculateSurveyProgress(payload);
  deriveServiceRuntimeState(next);
  recalcCrewTotals(next);
  recalcPowerAvailable(next);

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function advanceTicks(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsAdvanceTicksPayload,
): GameActionResponse<DevtoolsAdvanceTicksRejectionCode> {
  if (!Number.isInteger(payload.count) || payload.count < 1 || payload.count > 240) {
    return failure(fixtureName, state, 'invalid_range');
  }

  const next = cloneFixtureState(state);
  next.tickCount += payload.count;

  replaceFixtureState(state, next);
  return success(fixtureName, state);
}

function resetToStarter(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
  payload: DevtoolsResetToStarterPayload,
): GameActionResponse<'invalid_state'> {
  void payload;

  const freshState = createFixtureState('starter');
  replaceFixtureState(state, freshState);
  return success(fixtureName, state);
}

function buildDevtoolsStateResponse(
  snapshot: GameCommandResponses['game_get_snapshot'],
  visible: boolean,
): RawDevtoolsGetStateResponse {
  return { visible, snapshot };
}

function buildDevtoolsVisibilityResponse(
  snapshot: GameCommandResponses['game_get_snapshot'],
  visible: boolean,
): RawDevtoolsSetVisibilityResponse {
  return { visible, snapshot };
}

function deriveServiceRuntimeState(state: PreviewFixtureState): void {
  const activeServiceCap =
    currentSystemLevelValue(state, 'logistics-spine', 'activeServiceSlots') ?? 0;
  let activeCount = 0;
  let reservedPower = 2;

  for (const service of state.services) {
    if (!service.desiredActive) {
      service.isActive = false;
      service.isPaused = false;
      service.pauseReason = null;
      continue;
    }

    if (service.assignedCrew < requiredCrewForService(service.id)) {
      service.isActive = false;
      service.isPaused = true;
      service.pauseReason = 'crew';
      continue;
    }

    if (activeCount >= activeServiceCap) {
      service.isActive = false;
      service.isPaused = true;
      service.pauseReason = 'capacity';
      continue;
    }

    const projectedReservedPower = reservedPower + servicePowerUpkeep(state, service.id);
    if (state.power.generated - projectedReservedPower < 0) {
      service.isActive = false;
      service.isPaused = true;
      service.pauseReason = 'deficit';
      continue;
    }

    service.isActive = true;
    service.isPaused = false;
    service.pauseReason = null;
    activeCount += 1;
    reservedPower = projectedReservedPower;
  }
}

function calculateSurveyProgress(payload: DevtoolsApplyProgressionPayload): number {
  const currentPlanetProgress = payload.surveyProgress['solstice-anchor'] ?? 0;

  if (payload.discoveredPlanets.includes('aurora-pier')) {
    return Math.max(
      PLANET_THRESHOLDS['aurora-pier'],
      payload.surveyProgress['aurora-pier'] ?? PLANET_THRESHOLDS['aurora-pier'],
    );
  }

  if (payload.discoveredPlanets.includes('cinder-forge')) {
    return (
      PLANET_THRESHOLDS['cinder-forge'] + (payload.surveyProgress['aurora-pier'] ?? 0)
    );
  }

  return payload.surveyProgress['cinder-forge'] ?? currentPlanetProgress;
}

function replaceFixtureState(target: PreviewFixtureState, next: PreviewFixtureState): void {
  target.tickCount = next.tickCount;
  target.stablePowerSeconds = next.stablePowerSeconds;
  target.activePlanetId = next.activePlanetId;
  target.discoveredPlanetIds = [...next.discoveredPlanetIds];
  target.doctrineIds = [...next.doctrineIds];
  target.doctrineFragments = next.doctrineFragments;
  target.surveyProgress = next.surveyProgress;
  target.materials = next.materials;
  target.data = next.data;
  target.power = { ...next.power };
  target.crew = { ...next.crew };
  target.systems = next.systems.map((system) => ({ ...system }));
  target.services = next.services.map((service) => ({ ...service }));
}

function sortByOrder<T extends string>(
  values: readonly T[],
  order: readonly T[],
): T[] {
  const orderIndex = new Map(order.map((value, index) => [value, index]));
  return [...values].sort(
    (left, right) => (orderIndex.get(left) ?? Number.MAX_SAFE_INTEGER) - (orderIndex.get(right) ?? Number.MAX_SAFE_INTEGER),
  );
}

function isNonNegativeNumber(value: number): boolean {
  return Number.isFinite(value) && value >= 0;
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
