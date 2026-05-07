import type {
  DevtoolsApplyServicesPayload,
  GameSnapshot,
  GatewayDevtoolsApplyServicesResponse,
  RawServiceStateSnapshot,
} from '$lib/game/api/types';

type ServicesGateway = {
  applyServices: (
    input: DevtoolsApplyServicesPayload,
  ) => Promise<GatewayDevtoolsApplyServicesResponse>;
};

type ServiceDraft = {
  id: RawServiceStateSnapshot['id'];
  desiredActive: boolean;
  assignedCrew: number | undefined;
  priority: number | undefined;
};

const ASSIGNED_CREW_MIN = 0;

export function createServicesPanelState(snapshot: GameSnapshot | null, gateway: ServicesGateway) {
  const initialDrafts = createDrafts(snapshot);
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let drafts = $state<ServiceDraft[]>(initialDrafts);
  let lastSeededDrafts = $state<ServiceDraft[]>(initialDrafts.map((draft) => ({ ...draft })));
  let hasSeededOnce = snapshot !== null;
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  const isDirty = $derived(hasServiceDraftChanges(drafts, lastSeededDrafts));

  function reseedDrafts(snapshot: GameSnapshot) {
    if (snapshot.services.length !== drafts.length) {
      drafts = createDrafts(snapshot);
      lastSeededDrafts = drafts.map((draft) => ({ ...draft }));
      return;
    }

    for (let index = 0; index < drafts.length; index++) {
      drafts[index].desiredActive = snapshot.services[index].desiredActive;
      drafts[index].assignedCrew = snapshot.services[index].assignedCrew;
      drafts[index].priority = snapshot.services[index].priority;
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
    if (!hasValidDrafts(drafts)) {
      errorMessage = 'invalid_range';
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await gateway.applyServices({
        services: drafts.map(({ id, desiredActive, assignedCrew, priority }) => ({
          id,
          desiredActive,
          assignedCrew: assignedCrew!,
          priority: priority!,
        })),
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

function createDrafts(snapshot: GameSnapshot | null): ServiceDraft[] {
  return (
    snapshot?.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
      id,
      desiredActive,
      assignedCrew,
      priority,
    })) ?? []
  );
}

function hasServiceDraftChanges(drafts: ServiceDraft[], lastSeededDrafts: ServiceDraft[]) {
  if (drafts.length !== lastSeededDrafts.length) {
    return drafts.length > 0 || lastSeededDrafts.length > 0;
  }

  return drafts.some((draft, index) => {
    const lastSeededDraft = lastSeededDrafts[index];
    return (
      !lastSeededDraft ||
      lastSeededDraft.id !== draft.id ||
      lastSeededDraft.desiredActive !== draft.desiredActive ||
      lastSeededDraft.assignedCrew !== draft.assignedCrew ||
      lastSeededDraft.priority !== draft.priority
    );
  });
}

function hasValidDrafts(drafts: ServiceDraft[]) {
  const serviceCount = drafts.length;

  return drafts.every(
    ({ assignedCrew, priority }) =>
      isAtLeast(assignedCrew, ASSIGNED_CREW_MIN) &&
      isInRange(priority, 1, Math.max(serviceCount, 1)),
  );
}

function isAtLeast(value: number | undefined, min: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min;
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
