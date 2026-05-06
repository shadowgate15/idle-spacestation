import type { GameSnapshot, GatewayDevtoolsApplyResourcesResponse } from '$lib/game/api/types';

type ResourcesGateway = {
  applyResources: (input: { materials: number; data: number }) => Promise<GatewayDevtoolsApplyResourcesResponse>;
};

const MATERIALS_MIN = 0;
const MATERIALS_MAX = 99999;
const DATA_MIN = 0;
const DATA_MAX = 99999;

export function createResourcesPanelState(snapshot: GameSnapshot | null, gateway: ResourcesGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let materialsDraft = $state<number | undefined>(snapshot?.resources.materials ?? 0);
  let dataDraft = $state<number | undefined>(snapshot?.resources.data ?? 0);
  let lastSeededMaterials = $state<number>(snapshot?.resources.materials ?? 0);
  let lastSeededData = $state<number>(snapshot?.resources.data ?? 0);
  let hasSeededOnce = snapshot !== null;
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  const isDirty = $derived(
    materialsDraft !== lastSeededMaterials || dataDraft !== lastSeededData,
  );

  function reseedDrafts(next: GameSnapshot) {
    materialsDraft = next.resources.materials;
    dataDraft = next.resources.data;
    lastSeededMaterials = next.resources.materials;
    lastSeededData = next.resources.data;
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
    if (
      !isInRange(materialsDraft, MATERIALS_MIN, MATERIALS_MAX) ||
      !isInRange(dataDraft, DATA_MIN, DATA_MAX)
    ) {
      errorMessage = 'invalid_range';
      if (currentSnapshot) {
        reseedDrafts(currentSnapshot);
      }
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyResources({
        materials: materialsDraft,
        data: dataDraft,
      });

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
    get materialsDraft() {
      return materialsDraft;
    },
    set materialsDraft(value: number | undefined) {
      materialsDraft = value;
    },
    get dataDraft() {
      return dataDraft;
    },
    set dataDraft(value: number | undefined) {
      dataDraft = value;
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
