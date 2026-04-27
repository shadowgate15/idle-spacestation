import type {
  DevtoolsApplySystemsPayload,
  GameSnapshot,
  GatewayDevtoolsApplySystemsResponse,
  RawSystemStateSnapshot,
} from '$lib/game/api/types';

type SystemsGateway = {
  applySystems: (input: DevtoolsApplySystemsPayload) => Promise<GatewayDevtoolsApplySystemsResponse>;
};

type SystemDraft = {
  id: RawSystemStateSnapshot['id'];
  level: number | undefined;
};

const SYSTEM_LEVEL_MIN = 1;
const SYSTEM_LEVEL_MAX = 4;

export function createSystemsPanelState(snapshot: GameSnapshot | null, gateway: SystemsGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let drafts = $state<SystemDraft[]>(createDrafts(snapshot));
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  let isDirty = $derived(hasDraftChanges(drafts, currentSnapshot));

  function sync(snapshot: GameSnapshot | null, force = false) {
    const wasDirty = hasDraftChanges(drafts, currentSnapshot);

    currentSnapshot = snapshot;

    if (!force && wasDirty) {
      return;
    }

    drafts = createDrafts(snapshot);

    if (snapshot) {
      errorMessage = null;
    }
  }

  async function apply() {
    if (!hasValidDraftLevels(drafts)) {
      errorMessage = 'invalid_range';
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applySystems({
        systems: drafts.map(({ id, level }) => ({ id, level: level! })),
      });

      currentSnapshot = response.snapshot;

      if (response.ok) {
        drafts = createDrafts(response.snapshot);
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
    get drafts() {
      return drafts;
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
    sync,
    apply,
  };
}

function createDrafts(snapshot: GameSnapshot | null): SystemDraft[] {
  return snapshot?.systems.map(({ id, level }) => ({ id, level })) ?? [];
}

function hasDraftChanges(drafts: SystemDraft[], snapshot: GameSnapshot | null) {
  const systems = snapshot?.systems ?? [];

  if (drafts.length !== systems.length) {
    return drafts.length > 0 || systems.length > 0;
  }

  return drafts.some((draft, index) => {
    const current = systems[index];
    return !current || current.id !== draft.id || current.level !== draft.level;
  });
}

function hasValidDraftLevels(drafts: SystemDraft[]) {
  return drafts.every(({ level }) => isInRange(level, SYSTEM_LEVEL_MIN, SYSTEM_LEVEL_MAX));
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
