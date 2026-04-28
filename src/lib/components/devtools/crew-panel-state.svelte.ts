import type { GameSnapshot, GatewayDevtoolsApplyCrewResponse } from '$lib/game/api/types';

type CrewGateway = {
  applyCrew: (input: { crewTotal: number }) => Promise<GatewayDevtoolsApplyCrewResponse>;
};

const CREW_MIN = 0;
const CREW_MAX = 999;

export function createCrewPanelState(snapshot: GameSnapshot | null, gateway: CrewGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let crewTotalDraft = $state<number | undefined>(snapshot?.resources.crew.total ?? 0);
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  let isDirty = $derived(crewTotalDraft !== (currentSnapshot?.resources.crew.total ?? 0));

  function sync(snapshot: GameSnapshot | null, force = false) {
    const previousCrewTotal = currentSnapshot?.resources.crew.total ?? 0;
    const wasDirty = crewTotalDraft !== previousCrewTotal;

    currentSnapshot = snapshot;

    if (!force && wasDirty) {
      return;
    }

    crewTotalDraft = snapshot?.resources.crew.total ?? 0;

    if (snapshot) {
      errorMessage = null;
    }
  }

  async function apply() {
    if (!isInRange(crewTotalDraft, CREW_MIN, CREW_MAX)) {
      errorMessage = 'invalid_range';
      crewTotalDraft = currentSnapshot?.resources.crew.total ?? 0;
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyCrew({ crewTotal: crewTotalDraft });

      currentSnapshot = response.snapshot;

      if (response.ok) {
        crewTotalDraft = response.snapshot.resources.crew.total;
        return;
      }

      crewTotalDraft = response.snapshot.resources.crew.total;
      errorMessage = response.reasonCode;
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
