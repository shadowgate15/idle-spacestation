import type {
  DevtoolsApplyProgressionPayload,
  DoctrineId,
  GameSnapshot,
  GatewayDevtoolsApplyProgressionResponse,
  PlanetId,
} from '$lib/game/api/types';

type ProgressionGateway = {
  applyProgression: (input: DevtoolsApplyProgressionPayload) => Promise<GatewayDevtoolsApplyProgressionResponse>;
};

type ProgressionDraft = {
  doctrineFragments: number | undefined;
  unlockedDoctrines: DoctrineId[];
  discoveredPlanets: PlanetId[];
  activePlanet: PlanetId;
  surveyProgress: number | undefined;
};

export const doctrineIds: DoctrineId[] = [
  'efficient-shifts',
  'deep-survey-protocols',
  'hardened-relays',
  'frontier-charters',
];

export const planetIds: PlanetId[] = ['solstice-anchor', 'cinder-forge', 'aurora-pier'];

const STARTER_PLANET_ID: PlanetId = 'solstice-anchor';
const SURVEY_PROGRESS_THRESHOLDS: Record<Exclude<PlanetId, typeof STARTER_PLANET_ID>, number> = {
  'cinder-forge': 600,
  'aurora-pier': 1400,
};

export function createProgressionPanelState(snapshot: GameSnapshot | null, gateway: ProgressionGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let draft = $state<ProgressionDraft>(createDraft(snapshot));
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  let activePlanetOptions = $derived(planetIds.filter((id) => draft.discoveredPlanets.includes(id)));
  let isDirty = $derived(hasDraftChanges(draft, currentSnapshot));

  function sync(snapshot: GameSnapshot | null, force = false) {
    const wasDirty = hasDraftChanges(draft, currentSnapshot);

    currentSnapshot = snapshot;

    if (!force && wasDirty) {
      return;
    }

    draft = createDraft(snapshot);

    if (snapshot) {
      errorMessage = null;
    }
  }

  function setDoctrineFragments(value: number | undefined) {
    draft.doctrineFragments = value;
  }

  function setSurveyProgress(value: number | undefined) {
    draft.surveyProgress = value;
  }

  function toggleUnlockedDoctrine(id: DoctrineId, checked: boolean) {
    draft.unlockedDoctrines = normalizeDoctrineIds(
      checked ? [...draft.unlockedDoctrines, id] : draft.unlockedDoctrines.filter((currentId) => currentId !== id),
    );
  }

  function toggleDiscoveredPlanet(id: PlanetId, checked: boolean) {
    if (id === STARTER_PLANET_ID && !checked) {
      return;
    }

    draft.discoveredPlanets = normalizePlanetIds(
      checked ? [...draft.discoveredPlanets, id] : draft.discoveredPlanets.filter((currentId) => currentId !== id),
    );

    if (!draft.discoveredPlanets.includes(draft.activePlanet)) {
      draft.activePlanet = draft.discoveredPlanets[0] ?? STARTER_PLANET_ID;
    }
  }

  function setActivePlanet(id: PlanetId) {
    if (draft.discoveredPlanets.includes(id)) {
      draft.activePlanet = id;
    }
  }

  async function apply() {
    if (!hasValidDraft(draft)) {
      errorMessage = 'invalid_range';
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyProgression({
        doctrineFragments: draft.doctrineFragments,
        unlockedDoctrines: normalizeDoctrineIds(draft.unlockedDoctrines),
        discoveredPlanets: normalizePlanetIds(draft.discoveredPlanets),
        activePlanet: draft.activePlanet,
        surveyProgress: {
          [draft.activePlanet]: draft.surveyProgress,
        },
      });

      currentSnapshot = response.snapshot;

      if (response.ok) {
        draft = createDraft(response.snapshot);
        return;
      }

      errorMessage = response.reasonCode;
    } finally {
      isApplying = false;
    }
  }

  return {
    get snapshot() {
      return currentSnapshot;
    },
    get draft() {
      return draft;
    },
    get errorMessage() {
      return errorMessage;
    },
    get isApplying() {
      return isApplying;
    },
    get isDirty() {
      return isDirty;
    },
    get activePlanetOptions() {
      return activePlanetOptions;
    },
    sync,
    setDoctrineFragments,
    setSurveyProgress,
    toggleUnlockedDoctrine,
    toggleDiscoveredPlanet,
    setActivePlanet,
    apply,
  };
}

function createDraft(snapshot: GameSnapshot | null): ProgressionDraft {
  const discoveredPlanets = normalizePlanetIds(snapshot?.run.discoveredPlanetIds ?? [STARTER_PLANET_ID]);
  const activePlanet = discoveredPlanets.includes(snapshot?.run.activePlanetId ?? STARTER_PLANET_ID)
    ? (snapshot?.run.activePlanetId ?? STARTER_PLANET_ID)
    : discoveredPlanets[0] ?? STARTER_PLANET_ID;

  return {
    doctrineFragments: snapshot?.run.doctrineFragments ?? 0,
    unlockedDoctrines: normalizeDoctrineIds(snapshot?.run.doctrineIds ?? []),
    discoveredPlanets,
    activePlanet,
    surveyProgress: deriveSurveyProgressDraft(snapshot, activePlanet),
  };
}

function deriveSurveyProgressDraft(snapshot: GameSnapshot | null, activePlanet: PlanetId) {
  if (!snapshot || activePlanet === STARTER_PLANET_ID) {
    return 0;
  }

  const threshold = SURVEY_PROGRESS_THRESHOLDS[activePlanet as Exclude<PlanetId, typeof STARTER_PLANET_ID>];
  if (!threshold) {
    return 0;
  }

  return clampSurveyProgress(snapshot.run.surveyProgress / threshold);
}

function clampSurveyProgress(value: number) {
  if (!Number.isFinite(value)) {
    return 0;
  }

  return Math.min(1, Math.max(0, value));
}

function hasDraftChanges(draft: ProgressionDraft, snapshot: GameSnapshot | null) {
  const currentDraft = createDraft(snapshot);

  return (
    draft.doctrineFragments !== currentDraft.doctrineFragments ||
    draft.activePlanet !== currentDraft.activePlanet ||
    draft.surveyProgress !== currentDraft.surveyProgress ||
    !hasSameIds(draft.unlockedDoctrines, currentDraft.unlockedDoctrines) ||
    !hasSameIds(draft.discoveredPlanets, currentDraft.discoveredPlanets)
  );
}

function hasValidDraft(draft: ProgressionDraft): draft is {
  doctrineFragments: number;
  unlockedDoctrines: DoctrineId[];
  discoveredPlanets: PlanetId[];
  activePlanet: PlanetId;
  surveyProgress: number;
} {
  return (
    isAtLeast(draft.doctrineFragments, 0) &&
    isInRange(draft.surveyProgress, 0, 1) &&
    hasSameIds(draft.unlockedDoctrines, normalizeDoctrineIds(draft.unlockedDoctrines)) &&
    hasSameIds(draft.discoveredPlanets, normalizePlanetIds(draft.discoveredPlanets)) &&
    draft.discoveredPlanets.includes(STARTER_PLANET_ID) &&
    draft.discoveredPlanets.includes(draft.activePlanet)
  );
}

function normalizeDoctrineIds(ids: DoctrineId[]) {
  return doctrineIds.filter((id) => ids.includes(id));
}

function normalizePlanetIds(ids: PlanetId[]) {
  const discoveredPlanets = planetIds.filter((id) => id === STARTER_PLANET_ID || ids.includes(id));

  return discoveredPlanets.includes(STARTER_PLANET_ID)
    ? discoveredPlanets
    : [STARTER_PLANET_ID, ...discoveredPlanets];
}

function hasSameIds<T extends string>(left: T[], right: T[]) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function isAtLeast(value: number | undefined, min: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min;
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
