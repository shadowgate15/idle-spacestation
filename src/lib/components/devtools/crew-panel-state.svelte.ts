import type { GameSnapshot, GatewayDevtoolsApplyCrewResponse } from '$lib/game/api/types';

type CrewGateway = {
  applyCrew: (input: { crewTotal: number }) => Promise<GatewayDevtoolsApplyCrewResponse>;
};

const CREW_MIN = 0;
const CREW_MAX = 999;

export function createCrewPanelState(snapshot: GameSnapshot | null, gateway: CrewGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let crewTotalDraft = $state<number | undefined>(snapshot?.resources.crew.total ?? 0);
  let lastSeededCrewTotal = $state<number>(snapshot?.resources.crew.total ?? 0);
  let hasSeededOnce = snapshot !== null;
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  const isDirty = $derived(crewTotalDraft !== lastSeededCrewTotal);

  function reseedDrafts(next: GameSnapshot) {
    crewTotalDraft = next.resources.crew.total;
    lastSeededCrewTotal = next.resources.crew.total;
  }

  function sync(snapshot: GameSnapshot | null) {
    currentSnapshot = snapshot;

    if (!hasSeededOnce && snapshot !== null) {
      reseedDrafts(snapshot);
      errorMessage = null;
      hasSeededOnce = true;
    }
  }

  async function apply() {
    if (!isInRange(crewTotalDraft, CREW_MIN, CREW_MAX)) {
      errorMessage = 'invalid_range';
      if (currentSnapshot) {
        reseedDrafts(currentSnapshot);
      }
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyCrew({ crewTotal: crewTotalDraft });

      currentSnapshot = response.snapshot;
      reseedDrafts(response.snapshot);

      if (!response.ok) {
        errorMessage = response.reasonCode;
      }
    } finally {
      isApplying = false;
    }
  }

  return {
    get snapshot() {
      return currentSnapshot;
    },
    get crewTotalDraft() {
      return crewTotalDraft;
    },
    set crewTotalDraft(value: number | undefined) {
      crewTotalDraft = value;
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

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
