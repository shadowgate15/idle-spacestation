import type {
  DevtoolsAdvanceTicksPayload,
  DevtoolsResetToStarterPayload,
  GameSnapshot,
  GatewayDevtoolsAdvanceTicksResponse,
  GatewayDevtoolsResetToStarterResponse,
} from '$lib/game/api/types';

type SessionGateway = {
  advanceTicks: (input: DevtoolsAdvanceTicksPayload) => Promise<GatewayDevtoolsAdvanceTicksResponse>;
  resetToStarter: (input: DevtoolsResetToStarterPayload) => Promise<GatewayDevtoolsResetToStarterResponse>;
};

const ADVANCE_TICKS_MIN = 1;
const ADVANCE_TICKS_MAX = 240;

export function createSessionPanelState(snapshot: GameSnapshot | null, gateway: SessionGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let advanceCount = $state<number | undefined>(ADVANCE_TICKS_MIN);
  let errorMessage = $state<string | null>(null);
  let isAdvancing = $state(false);
  let isResetting = $state(false);
  let isConfirmingReset = $state(false);

  let isBusy = $derived(isAdvancing || isResetting);

  function sync(snapshot: GameSnapshot | null) {
    currentSnapshot = snapshot;

    if (snapshot) {
      errorMessage = null;
    }
  }

  function setAdvanceCount(value: number | undefined) {
    advanceCount = value;
  }

  function requestResetConfirmation() {
    isConfirmingReset = true;
  }

  function cancelResetConfirmation() {
    isConfirmingReset = false;
  }

  async function advance() {
    if (!isInRange(advanceCount, ADVANCE_TICKS_MIN, ADVANCE_TICKS_MAX)) {
      errorMessage = 'invalid_range';
      return;
    }

    isAdvancing = true;
    errorMessage = null;

    try {
      const response = await gateway.advanceTicks({ count: advanceCount });

      currentSnapshot = response.snapshot;

      if (response.ok) {
        return;
      }

      errorMessage = response.reasonCode;
    } finally {
      isAdvancing = false;
    }
  }

  async function confirmReset() {
    isResetting = true;
    errorMessage = null;

    try {
      const response = await gateway.resetToStarter({});

      currentSnapshot = response.snapshot;
      isConfirmingReset = false;

      if (response.ok) {
        return;
      }

      errorMessage = response.reasonCode;
    } finally {
      isResetting = false;
    }
  }

  return {
    get snapshot() {
      return currentSnapshot;
    },
    get advanceCount() {
      return advanceCount;
    },
    get errorMessage() {
      return errorMessage;
    },
    get isAdvancing() {
      return isAdvancing;
    },
    get isResetting() {
      return isResetting;
    },
    get isBusy() {
      return isBusy;
    },
    get isConfirmingReset() {
      return isConfirmingReset;
    },
    sync,
    setAdvanceCount,
    requestResetConfirmation,
    cancelResetConfirmation,
    advance,
    confirmReset,
  };
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
