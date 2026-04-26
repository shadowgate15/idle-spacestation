import type {
  DoctrineId,
  DoctrinePurchaseRejectionCode,
  DoctrinePurchaseOptionSnapshot,
  DoctrineSnapshot,
  GuidanceTrigger,
  PlanetId,
  PlanetModifierSnapshot,
  PlanetsRouteSnapshot,
  PrestigeReasonCode,
  PrestigeRouteSnapshot,
  PreviewFixtureName,
  PreviewFixtureState,
  RawGameSnapshot,
  RawServiceStateSnapshot,
  RawSystemStateSnapshot,
  ResetConsequenceSnapshot,
  ResourceDeltaSnapshot,
  ServiceFamily,
  ServiceRouteEntrySnapshot,
  ServicesRouteSnapshot,
  SurveyProgressSnapshot,
  SystemId,
  SystemsRouteSnapshot,
  SystemRouteEntrySnapshot,
  WarningSnapshot,
} from '../types';

export const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';

const HOUSEKEEPING_POWER = 2;
const PRESTIGE_REQUIRED_STABLE_SECONDS = 300;

type SystemProgression = {
  description: string;
  levels: Array<Record<string, number | null>>;
};

type PlanetDefinition = {
  id: PlanetId;
  name: string;
  description: string;
  modifiers: PlanetModifierSnapshot[];
};

type ServiceDefinition = {
  id: RawServiceStateSnapshot['id'];
  name: string;
  description: string;
  family: ServiceFamily;
  requiredCrew: number;
  powerUpkeep: number;
  powerOutput: number;
  materialsDelta: number;
  dataDelta: number;
  surveyDelta: number;
};

const planetDefinitions: Record<PlanetId, PlanetDefinition> = {
  'solstice-anchor': {
    id: 'solstice-anchor',
    name: 'Solstice Anchor',
    description: 'Starter balanced planet with efficient crews but weaker research output.',
    modifiers: [
      {
        target: 'crew-efficiency',
        label: 'Crew efficiency',
        percent: 0.1,
        effectText: '+10% Crew efficiency',
      },
      {
        target: 'data-output',
        label: 'Data output',
        percent: -0.1,
        effectText: '-10% Data output',
      },
    ],
  },
  'cinder-forge': {
    id: 'cinder-forge',
    name: 'Cinder Forge',
    description: 'Industrial planet tuned for material throughput at higher power cost.',
    modifiers: [
      {
        target: 'materials-output',
        label: 'Materials output',
        percent: 0.25,
        effectText: '+25% Materials output',
      },
      {
        target: 'service-power-upkeep',
        label: 'Service power upkeep',
        percent: 0.2,
        effectText: '+20% Service power upkeep',
      },
    ],
  },
  'aurora-pier': {
    id: 'aurora-pier',
    name: 'Aurora Pier',
    description: 'Research planet with stronger data returns and lower crew capacity.',
    modifiers: [
      {
        target: 'data-output',
        label: 'Data output',
        percent: 0.3,
        effectText: '+30% Data output',
      },
      {
        target: 'crew-capacity',
        label: 'Crew capacity',
        percent: -0.15,
        effectText: '-15% Crew capacity',
      },
    ],
  },
};

const planetOrder: PlanetId[] = ['solstice-anchor', 'cinder-forge', 'aurora-pier'];

const planetSurveyThresholds: Record<Exclude<PlanetId, 'solstice-anchor'>, number> = {
  'cinder-forge': 600,
  'aurora-pier': 1400,
};

const systemProgressions: Record<SystemId, SystemProgression> = {
  'reactor-core': {
    description: 'Defines baseline power throughput and service power cap.',
    levels: [
      { powerOutput: 8, servicePowerCap: 8, upgradeCostMaterials: 40 },
      { powerOutput: 12, servicePowerCap: 12, upgradeCostMaterials: 80 },
      { powerOutput: 16, servicePowerCap: 16, upgradeCostMaterials: 140 },
      { powerOutput: 20, servicePowerCap: 20, upgradeCostMaterials: null },
    ],
  },
  'habitat-ring': {
    description: 'Defines crew capacity and recovery ceiling.',
    levels: [
      { crewCapacity: 6, recoveryCeilingPerMinute: 1, upgradeCostMaterials: 35 },
      { crewCapacity: 8, recoveryCeilingPerMinute: 1.5, upgradeCostMaterials: 75 },
      { crewCapacity: 10, recoveryCeilingPerMinute: 2, upgradeCostMaterials: 130 },
      { crewCapacity: 12, recoveryCeilingPerMinute: 2.5, upgradeCostMaterials: null },
    ],
  },
  'logistics-spine': {
    description: 'Defines active service slots and materials stockpile cap.',
    levels: [
      { activeServiceSlots: 2, materialsCapacity: 250, upgradeCostMaterials: 30 },
      { activeServiceSlots: 3, materialsCapacity: 400, upgradeCostMaterials: 70 },
      { activeServiceSlots: 4, materialsCapacity: 600, upgradeCostMaterials: 120 },
      { activeServiceSlots: 5, materialsCapacity: 850, upgradeCostMaterials: null },
    ],
  },
  'survey-array': {
    description: 'Defines data and survey multipliers for discovery progress.',
    levels: [
      { dataMultiplier: 1, surveyMultiplier: 1, upgradeCostMaterials: 50 },
      { dataMultiplier: 1.2, surveyMultiplier: 1.15, upgradeCostMaterials: 95 },
      { dataMultiplier: 1.4, surveyMultiplier: 1.3, upgradeCostMaterials: 155 },
      { dataMultiplier: 1.65, surveyMultiplier: 1.5, upgradeCostMaterials: null },
    ],
  },
};

const systemOrder: SystemId[] = ['reactor-core', 'habitat-ring', 'logistics-spine', 'survey-array'];

const serviceDefinitions: Record<RawServiceStateSnapshot['id'], ServiceDefinition> = {
  'solar-harvester': {
    id: 'solar-harvester',
    name: 'Solar Harvester',
    description: 'Primary renewable power source for early station operations.',
    family: 'production',
    requiredCrew: 2,
    powerUpkeep: 0,
    powerOutput: 4,
    materialsDelta: 0,
    dataDelta: 0,
    surveyDelta: 0,
  },
  'ore-reclaimer': {
    id: 'ore-reclaimer',
    name: 'Ore Reclaimer',
    description: 'Consumes station capacity to turn scrap flow into materials.',
    family: 'production',
    requiredCrew: 1,
    powerUpkeep: 3,
    powerOutput: 0,
    materialsDelta: 2,
    dataDelta: 0,
    surveyDelta: 0,
  },
  'survey-uplink': {
    id: 'survey-uplink',
    name: 'Survey Uplink',
    description: 'Builds survey progress and trickles research data.',
    family: 'production',
    requiredCrew: 1,
    powerUpkeep: 2,
    powerOutput: 0,
    materialsDelta: 0,
    dataDelta: 1.5,
    surveyDelta: 1,
  },
  'maintenance-bay': {
    id: 'maintenance-bay',
    name: 'Maintenance Bay',
    description: 'Reduces global service power upkeep pressure.',
    family: 'support',
    requiredCrew: 1,
    powerUpkeep: 1,
    powerOutput: 0,
    materialsDelta: 0,
    dataDelta: 0,
    surveyDelta: 0,
  },
  'command-relay': {
    id: 'command-relay',
    name: 'Command Relay',
    description: 'Stabilizes priority handling and increases survey speed.',
    family: 'command',
    requiredCrew: 1,
    powerUpkeep: 1,
    powerOutput: 0,
    materialsDelta: 0,
    dataDelta: 0,
    surveyDelta: 0,
  },
  'fabrication-loop': {
    id: 'fabrication-loop',
    name: 'Fabrication Loop',
    description: 'Converts materials into research data.',
    family: 'conversion',
    requiredCrew: 1,
    powerUpkeep: 2,
    powerOutput: 0,
    materialsDelta: -1.5,
    dataDelta: 2,
    surveyDelta: 0,
  },
};

const doctrineDefinitions: Record<DoctrineId, DoctrineSnapshot> = {
  'efficient-shifts': {
    id: 'efficient-shifts',
    name: 'Efficient Shifts',
    description: 'The first support service needs 1 less Crew, to a minimum of 1.',
  },
  'deep-survey-protocols': {
    id: 'deep-survey-protocols',
    name: 'Deep Survey Protocols',
    description: 'Survey Uplink grants 20% more survey progress.',
  },
  'hardened-relays': {
    id: 'hardened-relays',
    name: 'Hardened Relays',
    description: 'Disabled services refund 50% of current-tick power upkeep back to the same tick.',
  },
  'frontier-charters': {
    id: 'frontier-charters',
    name: 'Frontier Charters',
    description: 'Newly discovered planets begin with Reactor Core level 2.',
  },
};

const doctrineOrder: DoctrineId[] = [
  'efficient-shifts',
  'deep-survey-protocols',
  'hardened-relays',
  'frontier-charters',
];

const resetConsequences: ResetConsequenceSnapshot[] = [
  {
    label: 'Discovered planets',
    outcome: 'retain',
    summary: 'Unlocked planets remain selectable for future runs.',
  },
  {
    label: 'Unlocked doctrines',
    outcome: 'retain',
    summary: 'Doctrine unlocks and spent fragments persist.',
  },
  {
    label: 'Doctrine fragments',
    outcome: 'retain',
    summary: 'Current fragment balance carries into the next run.',
  },
  {
    label: 'Lifetime stats',
    outcome: 'retain',
    summary: 'Lifetime ticks, prestiges, and best pace remain in the profile.',
  },
  {
    label: 'Materials and Data',
    outcome: 'reset',
    summary: 'Run stockpiles return to fresh-profile values.',
  },
  {
    label: 'Services and assignments',
    outcome: 'reset',
    summary: 'All services return to inactive with no Crew assigned.',
  },
  {
    label: 'System levels and survey progress',
    outcome: 'reset',
    summary: 'System upgrades and current survey progress are cleared.',
  },
];

const baseStarterServices = (): RawServiceStateSnapshot[] => [
  {
    id: 'solar-harvester',
    desiredActive: true,
    isActive: true,
    isPaused: false,
    pauseReason: null,
    priority: 1,
    assignedCrew: 2,
  },
  {
    id: 'ore-reclaimer',
    desiredActive: false,
    isActive: false,
    isPaused: false,
    pauseReason: null,
    priority: 2,
    assignedCrew: 0,
  },
  {
    id: 'survey-uplink',
    desiredActive: false,
    isActive: false,
    isPaused: false,
    pauseReason: null,
    priority: 3,
    assignedCrew: 0,
  },
  {
    id: 'maintenance-bay',
    desiredActive: false,
    isActive: false,
    isPaused: false,
    pauseReason: null,
    priority: 4,
    assignedCrew: 0,
  },
  {
    id: 'command-relay',
    desiredActive: false,
    isActive: false,
    isPaused: false,
    pauseReason: null,
    priority: 5,
    assignedCrew: 0,
  },
  {
    id: 'fabrication-loop',
    desiredActive: false,
    isActive: false,
    isPaused: false,
    pauseReason: null,
    priority: 6,
    assignedCrew: 0,
  },
];

const baseStarterSystems = (): RawSystemStateSnapshot[] => [
  { id: 'reactor-core', level: 1 },
  { id: 'habitat-ring', level: 1 },
  { id: 'logistics-spine', level: 1 },
  { id: 'survey-array', level: 1 },
];

export const starter: PreviewFixtureState = {
  tickCount: 0,
  stablePowerSeconds: 0,
  activePlanetId: 'solstice-anchor',
  discoveredPlanetIds: ['solstice-anchor'],
  doctrineIds: [],
  doctrineFragments: 0,
  surveyProgress: 0,
  materials: 120,
  data: 0,
  power: {
    generated: 8,
    reserved: 2,
    available: 6,
  },
  crew: {
    total: 6,
    assigned: 2,
    available: 4,
  },
  systems: baseStarterSystems(),
  services: baseStarterServices(),
};

export const deficit: PreviewFixtureState = {
  tickCount: 240,
  stablePowerSeconds: 0,
  activePlanetId: 'solstice-anchor',
  discoveredPlanetIds: ['solstice-anchor'],
  doctrineIds: ['hardened-relays'],
  doctrineFragments: 1,
  surveyProgress: 420,
  materials: 64,
  data: 12,
  power: {
    generated: 8,
    reserved: 11,
    available: -3,
  },
  crew: {
    total: 6,
    assigned: 4,
    available: 2,
  },
  systems: baseStarterSystems(),
  services: [
    {
      id: 'solar-harvester',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 1,
      assignedCrew: 2,
    },
    {
      id: 'ore-reclaimer',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 2,
      assignedCrew: 1,
    },
    {
      id: 'survey-uplink',
      desiredActive: true,
      isActive: false,
      isPaused: true,
      pauseReason: 'deficit',
      priority: 3,
      assignedCrew: 0,
    },
    {
      id: 'maintenance-bay',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 4,
      assignedCrew: 1,
    },
    {
      id: 'command-relay',
      desiredActive: false,
      isActive: false,
      isPaused: false,
      pauseReason: null,
      priority: 5,
      assignedCrew: 0,
    },
    {
      id: 'fabrication-loop',
      desiredActive: false,
      isActive: false,
      isPaused: false,
      pauseReason: null,
      priority: 6,
      assignedCrew: 0,
    },
  ],
};

export const allPlanets: PreviewFixtureState = {
  tickCount: 2_400,
  stablePowerSeconds: 96,
  activePlanetId: 'cinder-forge',
  discoveredPlanetIds: ['aurora-pier', 'cinder-forge', 'solstice-anchor'],
  doctrineIds: ['frontier-charters'],
  doctrineFragments: 1,
  surveyProgress: 1_550,
  materials: 410,
  data: 92,
  power: {
    generated: 16,
    reserved: 8,
    available: 8,
  },
  crew: {
    total: 8,
    assigned: 5,
    available: 3,
  },
  systems: [
    { id: 'reactor-core', level: 3 },
    { id: 'habitat-ring', level: 2 },
    { id: 'logistics-spine', level: 3 },
    { id: 'survey-array', level: 2 },
  ],
  services: [
    {
      id: 'solar-harvester',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 1,
      assignedCrew: 2,
    },
    {
      id: 'ore-reclaimer',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 2,
      assignedCrew: 1,
    },
    {
      id: 'survey-uplink',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 3,
      assignedCrew: 1,
    },
    {
      id: 'maintenance-bay',
      desiredActive: false,
      isActive: false,
      isPaused: false,
      pauseReason: null,
      priority: 4,
      assignedCrew: 0,
    },
    {
      id: 'command-relay',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 5,
      assignedCrew: 1,
    },
    {
      id: 'fabrication-loop',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 6,
      assignedCrew: 1,
    },
  ],
};

export const prestigeReady: PreviewFixtureState = {
  tickCount: 4_800,
  stablePowerSeconds: 300,
  activePlanetId: 'aurora-pier',
  discoveredPlanetIds: ['aurora-pier', 'cinder-forge', 'solstice-anchor'],
  doctrineIds: ['efficient-shifts', 'deep-survey-protocols'],
  doctrineFragments: 2,
  surveyProgress: 1_550,
  materials: 520,
  data: 320,
  power: {
    generated: 20,
    reserved: 14,
    available: 6,
  },
  crew: {
    total: 10,
    assigned: 6,
    available: 4,
  },
  systems: [
    { id: 'reactor-core', level: 2 },
    { id: 'habitat-ring', level: 2 },
    { id: 'logistics-spine', level: 2 },
    { id: 'survey-array', level: 1 },
  ],
  services: [
    {
      id: 'solar-harvester',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 1,
      assignedCrew: 2,
    },
    {
      id: 'ore-reclaimer',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 2,
      assignedCrew: 1,
    },
    {
      id: 'survey-uplink',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 3,
      assignedCrew: 1,
    },
    {
      id: 'maintenance-bay',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 4,
      assignedCrew: 1,
    },
    {
      id: 'command-relay',
      desiredActive: true,
      isActive: true,
      isPaused: false,
      pauseReason: null,
      priority: 5,
      assignedCrew: 1,
    },
    {
      id: 'fabrication-loop',
      desiredActive: false,
      isActive: false,
      isPaused: false,
      pauseReason: null,
      priority: 6,
      assignedCrew: 0,
    },
  ],
};

export const fixtures: Record<PreviewFixtureName, PreviewFixtureState> = {
  starter,
  deficit,
  'all-planets': allPlanets,
  'prestige-ready': prestigeReady,
};

export function isPreviewFixtureName(
  value: string | null | undefined,
): value is PreviewFixtureName {
  return (
    value === 'starter' ||
    value === 'deficit' ||
    value === 'all-planets' ||
    value === 'prestige-ready'
  );
}

export function readPreviewFixtureName(
  storage: Pick<Storage, 'getItem'> | undefined,
): PreviewFixtureName | null {
  const selected = storage?.getItem(FIXTURE_STORAGE_KEY) ?? null;
  return isPreviewFixtureName(selected) ? selected : null;
}

export function cloneFixtureState(state: PreviewFixtureState): PreviewFixtureState {
  return {
    ...state,
    discoveredPlanetIds: [...state.discoveredPlanetIds],
    doctrineIds: [...state.doctrineIds],
    power: { ...state.power },
    crew: { ...state.crew },
    systems: state.systems.map((system) => ({ ...system })),
    services: state.services.map((service) => ({ ...service })),
  };
}

export function createFixtureState(name: PreviewFixtureName): PreviewFixtureState {
  return cloneFixtureState(fixtures[name]);
}

export function buildSnapshotFromFixtureState(
  state: PreviewFixtureState,
  fixtureName: PreviewFixtureName,
): RawGameSnapshot {
  const stationTier = calculateStationTier(state.systems);
  const routeSnapshots = {
    overview: buildOverviewRoute(state, stationTier),
    systems: buildSystemsRoute(state),
    services: buildServicesRoute(state),
    planets: buildPlanetsRoute(state),
    prestige: buildPrestigeRoute(state, stationTier),
  };

  return {
    meta: {
      source: 'preview-fixture',
      fixtureName,
      tickCount: state.tickCount,
    },
    run: {
      activePlanetId: state.activePlanetId,
      discoveredPlanetIds: [...state.discoveredPlanetIds],
      doctrineIds: [...state.doctrineIds],
      doctrineFragments: state.doctrineFragments,
      surveyProgress: state.surveyProgress,
      stationTier,
      stablePowerSeconds: state.stablePowerSeconds,
    },
    resources: {
      power: { ...state.power },
      materials: state.materials,
      data: state.data,
      crew: { ...state.crew },
    },
    systems: state.systems.map((system) => ({ ...system })),
    services: state.services.map((service) => ({ ...service })),
    routeSnapshots,
  };
}

function buildOverviewRoute(state: PreviewFixtureState, stationTier: number) {
  const activePlanet = planetDefinitions[state.activePlanetId];
  const deficitWarnings = buildDeficitWarnings(state);
  const serviceUtilization = buildServiceUtilization(state);

  const guidanceTriggers = new Set<GuidanceTrigger>(['review-station-status']);
  if (deficitWarnings.length > 0) {
    guidanceTriggers.add('clear-power-deficit');
  }
  if (stationTier < 4) {
    guidanceTriggers.add('upgrade-reactor-core');
  }
  if (serviceUtilization.active >= serviceUtilization.capacity) {
    guidanceTriggers.add('upgrade-logistics-spine');
  }
  if (!state.services.some((service) => service.id === 'survey-uplink' && service.isActive)) {
    guidanceTriggers.add('activate-survey-uplink');
  }
  if (state.discoveredPlanetIds.filter((planetId) => planetId !== 'solstice-anchor').length < 2) {
    guidanceTriggers.add('discover-second-planet');
  }
  if (state.doctrineFragments > 0) {
    guidanceTriggers.add('spend-doctrine-fragment');
  }
  if (buildPrestigeReasonCodes(state, stationTier).length === 0) {
    guidanceTriggers.add('prestige-now');
  }

  return {
    activePlanet: {
      id: activePlanet.id,
      name: activePlanet.name,
      description: activePlanet.description,
      modifiers: activePlanet.modifiers.map(clonePlanetModifier),
    },
    resourceDeltas: buildResourceDeltas(state),
    deficitWarnings,
    stationTier: {
      current: stationTier,
      max: 4,
      label: `Tier ${stationTier}`,
    },
    serviceUtilization,
    surveyProgress: buildSurveyProgress(state),
    guidanceTriggers: [...guidanceTriggers],
  };
}

function buildSystemsRoute(state: PreviewFixtureState): SystemsRouteSnapshot {
  return {
    systems: systemOrder.map((systemId) => buildSystemEntry(state, systemId)),
  };
}

function buildServicesRoute(state: PreviewFixtureState): ServicesRouteSnapshot {
  return {
    services: [...state.services]
      .sort((left, right) => left.priority - right.priority)
      .map((service) => buildServiceEntry(service)),
    utilization: buildServiceUtilization(state),
    deficitWarnings: buildDeficitWarnings(state),
  };
}

function buildPlanetsRoute(state: PreviewFixtureState): PlanetsRouteSnapshot {
  return {
    activePlanetId: state.activePlanetId,
    planets: planetOrder.map((planetId) => {
      const definition = planetDefinitions[planetId];
      const discovered = state.discoveredPlanetIds.includes(planetId);
      const active = state.activePlanetId === planetId;
      const surveyThreshold =
        planetId === 'solstice-anchor' ? null : planetSurveyThresholds[planetId];
      const selectableForNextRun = discovered && !active;
      const selectabilityReason = !discovered
        ? 'Survey progress has not reached this world yet.'
        : active
          ? 'Current run already operates on this planet.'
          : null;

      return {
        id: planetId,
        name: definition.name,
        description: definition.description,
        discovered,
        active,
        selectableForNextRun,
        selectabilityReason,
        modifiers: definition.modifiers.map(clonePlanetModifier),
        surveyThreshold,
        surveyProgress:
          surveyThreshold === null
            ? state.surveyProgress
            : Math.min(state.surveyProgress, surveyThreshold),
      };
    }),
    surveyProgress: buildSurveyProgress(state),
  };
}

function buildPrestigeRoute(
  state: PreviewFixtureState,
  stationTier: number,
): PrestigeRouteSnapshot {
  const reasonCodes = buildPrestigeReasonCodes(state, stationTier);

  return {
    eligibility: {
      eligible: reasonCodes.length === 0,
      reasonCodes,
      summary:
        reasonCodes.length === 0
          ? 'Prestige is available. Doctrine fragments and discovered planets will persist into the next run.'
          : `Prestige blocked: ${reasonCodes.join(', ')}.`,
      stablePowerSeconds: state.stablePowerSeconds,
      requiredStablePowerSeconds: PRESTIGE_REQUIRED_STABLE_SECONDS,
    },
    doctrineFragments: state.doctrineFragments,
    unlockedDoctrines: state.doctrineIds.map((id) => doctrineDefinitions[id]),
    purchaseOptions: doctrineOrder.map((doctrineId) =>
      buildDoctrinePurchaseOption(state, doctrineId),
    ),
    resetConsequences: resetConsequences.map((entry) => ({ ...entry })),
  };
}

function buildSystemEntry(
  state: PreviewFixtureState,
  systemId: SystemId,
): SystemRouteEntrySnapshot {
  const systemState = state.systems.find((system) => system.id === systemId) ?? {
    id: systemId,
    level: 1,
  };
  const progression = systemProgressions[systemId];
  const levelIndex = Math.max(0, Math.min(systemState.level - 1, progression.levels.length - 1));
  const level = progression.levels[levelIndex];
  const upgradeCostMaterials = level.upgradeCostMaterials as number | null;
  const canUpgrade = upgradeCostMaterials !== null && state.materials >= upgradeCostMaterials;
  const upgradeBlockedReason =
    upgradeCostMaterials === null
      ? 'Max level reached.'
      : canUpgrade
        ? null
        : `Needs ${upgradeCostMaterials} Materials.`;

  let capValues: SystemRouteEntrySnapshot['capValues'];

  switch (systemId) {
    case 'reactor-core':
      capValues = [
        {
          key: 'power-output',
          label: 'Power output',
          value: level.powerOutput as number,
          unit: 'power',
        },
        {
          key: 'service-power-cap',
          label: 'Service power cap',
          value: level.servicePowerCap as number,
          unit: 'power',
        },
      ];
      break;
    case 'habitat-ring':
      capValues = [
        {
          key: 'crew-capacity',
          label: 'Crew capacity',
          value: applyCrewCapacityPlanetModifier(
            state.activePlanetId,
            level.crewCapacity as number,
          ),
          unit: 'crew',
        },
        {
          key: 'crew-recovery',
          label: 'Crew recovery ceiling',
          value: level.recoveryCeilingPerMinute as number,
          unit: 'crew/min',
        },
      ];
      break;
    case 'logistics-spine':
      capValues = [
        {
          key: 'active-service-slots',
          label: 'Active service slots',
          value: level.activeServiceSlots as number,
          unit: 'slots',
        },
        {
          key: 'materials-capacity',
          label: 'Materials capacity',
          value: level.materialsCapacity as number,
          unit: 'materials',
        },
      ];
      break;
    case 'survey-array':
      capValues = [
        {
          key: 'data-multiplier',
          label: 'Data multiplier',
          value: level.dataMultiplier as number,
          unit: 'x',
        },
        {
          key: 'survey-multiplier',
          label: 'Survey multiplier',
          value: level.surveyMultiplier as number,
          unit: 'x',
        },
      ];
      break;
  }

  return {
    id: systemId,
    name: titleCase(systemId),
    description: progression.description,
    level: systemState.level,
    maxLevel: progression.levels.length,
    capValues,
    upgradeCostMaterials,
    canUpgrade,
    upgradeBlockedReason,
  };
}

function buildServiceEntry(service: RawServiceStateSnapshot): ServiceRouteEntrySnapshot {
  const definition = serviceDefinitions[service.id];
  const status = service.isActive
    ? 'active'
    : service.isPaused || service.desiredActive
      ? 'paused'
      : 'disabled';

  return {
    id: service.id,
    name: definition.name,
    description: definition.description,
    family: definition.family,
    priorityOrder: service.priority,
    status,
    statusLabel: titleCase(status),
    desiredActive: service.desiredActive,
    crewAssignment: {
      current: service.assignedCrew,
      required: definition.requiredCrew,
    },
    powerUsage: {
      upkeep: definition.powerUpkeep,
      output: definition.powerOutput,
    },
    disabledReasons: service.pauseReason ? [service.pauseReason] : [],
  };
}

function buildDoctrinePurchaseOption(
  state: PreviewFixtureState,
  doctrineId: DoctrineId,
): DoctrinePurchaseOptionSnapshot {
  const doctrine = doctrineDefinitions[doctrineId];
  const alreadyUnlocked = state.doctrineIds.includes(doctrineId);
  const blockedReason: DoctrinePurchaseRejectionCode | null = alreadyUnlocked
    ? 'already-unlocked'
    : state.doctrineFragments < 1
      ? 'insufficient-fragments'
      : null;

  return {
    ...doctrine,
    costFragments: 1,
    available: blockedReason === null,
    blockedReason,
  };
}

function buildResourceDeltas(state: PreviewFixtureState): ResourceDeltaSnapshot[] {
  const dataMultiplier = currentSystemLevelValue(state, 'survey-array', 'dataMultiplier') as number;
  const surveyMultiplier = currentSystemLevelValue(
    state,
    'survey-array',
    'surveyMultiplier',
  ) as number;
  const materialModifier = activePlanetPercent(state.activePlanetId, 'materials-output');
  const dataModifier = activePlanetPercent(state.activePlanetId, 'data-output');
  const activeServices = state.services.filter((service) => service.isActive);

  const grossPower = activeServices.reduce(
    (sum, service) => sum + serviceDefinitions[service.id].powerOutput,
    0,
  );
  const serviceUpkeep = activeServices.reduce(
    (sum, service) =>
      sum +
      applyPowerUpkeepPlanetModifier(
        state.activePlanetId,
        serviceDefinitions[service.id].powerUpkeep,
      ),
    0,
  );
  const materialsDelta = activeServices.reduce(
    (sum, service) =>
      sum + applyPositiveModifier(serviceDefinitions[service.id].materialsDelta, materialModifier),
    0,
  );
  const dataDelta = activeServices.reduce(
    (sum, service) =>
      sum +
      applyPositiveModifier(serviceDefinitions[service.id].dataDelta, dataModifier) *
        dataMultiplier,
    0,
  );
  const surveyDelta =
    activeServices.reduce((sum, service) => sum + serviceDefinitions[service.id].surveyDelta, 0) *
    surveyMultiplier;

  return [
    makeResourceDelta('power', 'Power', grossPower - serviceUpkeep - HOUSEKEEPING_POWER),
    makeResourceDelta('materials', 'Materials', materialsDelta),
    makeResourceDelta('data', 'Data', dataDelta),
    makeResourceDelta('crew', 'Survey progress', surveyDelta),
  ];
}

function buildDeficitWarnings(state: PreviewFixtureState): WarningSnapshot[] {
  const warnings: WarningSnapshot[] = [];
  const deficitServices = state.services.filter((service) => service.pauseReason === 'deficit');

  if (state.power.available < 0) {
    warnings.push({
      code: 'power-deficit',
      severity: 'critical',
      title: 'Power deficit in progress',
      body: `Reserve is ${Math.abs(state.power.available).toFixed(1)} below zero. Lower-priority services are being shed.`,
    });
  }

  if (deficitServices.length > 0) {
    warnings.push({
      code: 'services-paused-by-deficit',
      severity: 'warning',
      title: 'Services paused by deficit handling',
      body: deficitServices.map((service) => serviceDefinitions[service.id].name).join(', '),
    });
  }

  return warnings;
}

function buildServiceUtilization(state: PreviewFixtureState) {
  const active = state.services.filter((service) => service.isActive).length;
  const capacity = currentSystemLevelValue(
    state,
    'logistics-spine',
    'activeServiceSlots',
  ) as number;

  return {
    active,
    capacity,
    available: Math.max(capacity - active, 0),
    summary: `${active} of ${capacity} active service slots in use`,
  };
}

function buildSurveyProgress(state: PreviewFixtureState): SurveyProgressSnapshot {
  const nextPlanetId = planetOrder.find(
    (planetId) => planetId !== 'solstice-anchor' && !state.discoveredPlanetIds.includes(planetId),
  ) as Exclude<PlanetId, 'solstice-anchor'> | undefined;

  if (!nextPlanetId) {
    return {
      current: state.surveyProgress,
      nextThreshold: null,
      nextPlanetId: null,
      nextPlanetName: null,
      summary: 'All survey targets discovered.',
    };
  }

  const nextThreshold = planetSurveyThresholds[nextPlanetId];

  return {
    current: state.surveyProgress,
    nextThreshold,
    nextPlanetId,
    nextPlanetName: planetDefinitions[nextPlanetId].name,
    summary: `${planetDefinitions[nextPlanetId].name} unlocks at ${nextThreshold} survey progress.`,
  };
}

function buildPrestigeReasonCodes(
  state: PreviewFixtureState,
  stationTier: number,
): PrestigeReasonCode[] {
  const reasons: PrestigeReasonCode[] = [];
  if (stationTier < 4) {
    reasons.push('station-tier-below-four');
  }
  if (state.discoveredPlanetIds.filter((planetId) => planetId !== 'solstice-anchor').length < 2) {
    reasons.push('needs-two-non-starter-planets');
  }
  if (state.stablePowerSeconds < PRESTIGE_REQUIRED_STABLE_SECONDS) {
    reasons.push('unstable-net-power');
  }
  return reasons;
}

function calculateStationTier(systems: RawSystemStateSnapshot[]): number {
  return Math.max(1, Math.min(4, systems.reduce((sum, system) => sum + system.level, 0) - 3));
}

function currentSystemLevelValue(
  state: PreviewFixtureState,
  systemId: SystemId,
  key: string,
): number | null {
  const system = state.systems.find((candidate) => candidate.id === systemId) ?? {
    id: systemId,
    level: 1,
  };
  const levels = systemProgressions[systemId].levels;
  const level = levels[Math.min(Math.max(system.level, 1), levels.length) - 1];
  return (level[key] as number | null | undefined) ?? null;
}

export { currentSystemLevelValue };

function activePlanetPercent(
  activePlanetId: PlanetId,
  target: PlanetModifierSnapshot['target'],
): number {
  return (
    planetDefinitions[activePlanetId].modifiers.find((modifier) => modifier.target === target)
      ?.percent ?? 0
  );
}

function applyPositiveModifier(value: number, percent: number): number {
  if (value <= 0) {
    return value;
  }
  return Number((value * (1 + percent)).toFixed(2));
}

function applyPowerUpkeepPlanetModifier(activePlanetId: PlanetId, upkeep: number): number {
  return Number(
    (upkeep * (1 + activePlanetPercent(activePlanetId, 'service-power-upkeep'))).toFixed(2),
  );
}

function applyCrewCapacityPlanetModifier(activePlanetId: PlanetId, capacity: number): number {
  const modifier = activePlanetPercent(activePlanetId, 'crew-capacity');
  return Math.max(1, Math.floor(capacity * (1 + modifier)));
}

function makeResourceDelta(
  id: 'power' | 'materials' | 'data' | 'crew',
  label: string,
  deltaPerSecond: number,
): ResourceDeltaSnapshot {
  return {
    id,
    label,
    deltaPerSecond: Number(deltaPerSecond.toFixed(2)),
    trend: deltaPerSecond > 0 ? 'positive' : deltaPerSecond < 0 ? 'negative' : 'neutral',
  };
}

function clonePlanetModifier(modifier: PlanetModifierSnapshot): PlanetModifierSnapshot {
  return { ...modifier };
}

function titleCase(value: string): string {
  return value
    .split('-')
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(' ');
}
