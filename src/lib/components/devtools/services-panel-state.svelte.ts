import type {
  DevtoolsApplyServicesPayload,
  GameSnapshot,
  GatewayDevtoolsApplyServicesResponse,
  RawServiceStateSnapshot,
} from '$lib/game/api/types';

type ServicesGateway = {
  applyServices: (input: DevtoolsApplyServicesPayload) => Promise<GatewayDevtoolsApplyServicesResponse>;
};

type ServiceDraft = {
  id: RawServiceStateSnapshot['id'];
  desiredActive: boolean;
  assignedCrew: number | undefined;
  priority: number | undefined;
};

const ASSIGNED_CREW_MIN = 0;

export function createServicesPanelState(snapshot: GameSnapshot | null, gateway: ServicesGateway) {
  let currentSnapshot = $state<GameSnapshot | null>(snapshot);
  let drafts = $state<ServiceDraft[]>(createDrafts(snapshot));
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

function hasDraftChanges(drafts: ServiceDraft[], snapshot: GameSnapshot | null) {
  const services = snapshot?.services ?? [];

  if (drafts.length !== services.length) {
    return drafts.length > 0 || services.length > 0;
  }

  return drafts.some((draft, index) => {
    const current = services[index];
    return (
      !current ||
      current.id !== draft.id ||
      current.desiredActive !== draft.desiredActive ||
      current.assignedCrew !== draft.assignedCrew ||
      current.priority !== draft.priority
    );
  });
}

function hasValidDrafts(drafts: ServiceDraft[]) {
  const serviceCount = drafts.length;

  return drafts.every(
    ({ assignedCrew, priority }) =>
      isAtLeast(assignedCrew, ASSIGNED_CREW_MIN) && isInRange(priority, 1, Math.max(serviceCount, 1)),
  );
}

function isAtLeast(value: number | undefined, min: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min;
}

function isInRange(value: number | undefined, min: number, max: number): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}
