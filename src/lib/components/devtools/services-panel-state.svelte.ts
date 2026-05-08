import type {
  DevtoolsApplyServicesPayload,
  GameSnapshot,
  GatewayDevtoolsApplyServicesResponse,
  RawServiceStateSnapshot,
} from '$lib/game/api/types';
import { isAtLeast, isInRange } from '$lib/utils';
import { createApplyPanelState } from './_create-apply-panel-state.svelte';

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
  const base = createApplyPanelState<ServiceDraft[], GatewayDevtoolsApplyServicesResponse>(
    snapshot,
    {
      seedDraft: (s) =>
        s?.services.map(({ id, desiredActive, assignedCrew, priority }) => ({
          id,
          desiredActive,
          assignedCrew,
          priority,
        })) ?? [],
      cloneDraft: (d) => d.map((draft) => ({ ...draft })),
      isDirty: hasServiceDraftChanges,
      isValid: (drafts) => {
        const count = drafts.length;
        return drafts.every(
          ({ assignedCrew, priority }) =>
            isAtLeast(assignedCrew, ASSIGNED_CREW_MIN) &&
            isInRange(priority, 1, Math.max(count, 1)),
        );
      },
      applyToGateway: (drafts) =>
        gateway.applyServices({
          services: drafts.map(({ id, desiredActive, assignedCrew, priority }) => ({
            id,
            desiredActive,
            assignedCrew: assignedCrew!,
            priority: priority!,
          })),
        }),
    },
  );

  return {
    get snapshot() {
      return base.snapshot;
    },
    get drafts() {
      return base.draft;
    },
    get errorMessage() {
      return base.errorMessage;
    },
    get isApplying() {
      return base.isApplying;
    },
    get isDirty() {
      return base.isDirty;
    },
    sync: base.sync,
    apply: base.apply,
  };
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
