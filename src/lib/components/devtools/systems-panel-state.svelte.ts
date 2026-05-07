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
  const initialDrafts = createDrafts(snapshot);
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let drafts = $state<SystemDraft[]>(initialDrafts);
  let lastSeededDrafts = $state<SystemDraft[]>(initialDrafts.map((draft) => ({ ...draft })));
  let hasSeededOnce = snapshot !== null;
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  const isDirty = $derived(hasSystemDraftChanges(drafts, lastSeededDrafts));

  function reseedDrafts(snapshot: GameSnapshot) {
    if (snapshot.systems.length !== drafts.length) {
      drafts = createDrafts(snapshot);
      lastSeededDrafts = drafts.map((draft) => ({ ...draft }));
      return;
    }

    for (let index = 0; index < drafts.length; index++) {
      drafts[index].level = snapshot.systems[index].level;
    }

    lastSeededDrafts = drafts.map((draft) => ({ ...draft }));
  }

  function sync(snapshot: GameSnapshot | null) {
    currentSnapshot = snapshot;

    if (!hasSeededOnce && snapshot !== null) {
      drafts = createDrafts(snapshot);
      lastSeededDrafts = drafts.map((draft) => ({ ...draft }));
      hasSeededOnce = true;
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
        reseedDrafts(response.snapshot);
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

function hasSystemDraftChanges(drafts: SystemDraft[], lastSeededDrafts: SystemDraft[]) {
  if (drafts.length !== lastSeededDrafts.length) {
    return drafts.length > 0 || lastSeededDrafts.length > 0;
  }

  return drafts.some((draft, index) => {
    const lastSeededDraft = lastSeededDrafts[index];
    return (
      !lastSeededDraft ||
      lastSeededDraft.id !== draft.id ||
      lastSeededDraft.level !== draft.level
    );
  });
}

function hasValidDraftLevels(drafts: SystemDraft[]) {
  return drafts.every(({ level }) => isInRange(level, SYSTEM_LEVEL_MIN, SYSTEM_LEVEL_MAX));
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
