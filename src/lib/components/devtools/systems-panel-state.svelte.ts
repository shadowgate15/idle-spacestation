import type {
  DevtoolsApplySystemsPayload,
  GameSnapshot,
  GatewayDevtoolsApplySystemsResponse,
  RawSystemStateSnapshot,
} from '$lib/game/api/types';
import { isInRange } from '$lib/utils';
import { createApplyPanelState } from './_create-apply-panel-state.svelte';

type SystemsGateway = {
  applySystems: (
    input: DevtoolsApplySystemsPayload,
  ) => Promise<GatewayDevtoolsApplySystemsResponse>;
};

type SystemDraft = {
  id: RawSystemStateSnapshot['id'];
  level: number | undefined;
};

const SYSTEM_LEVEL_MIN = 1;
const SYSTEM_LEVEL_MAX = 4;

export function createSystemsPanelState(snapshot: GameSnapshot | null, gateway: SystemsGateway) {
  const base = createApplyPanelState<SystemDraft[], GatewayDevtoolsApplySystemsResponse>(snapshot, {
    seedDraft: (s) => s?.systems.map(({ id, level }) => ({ id, level })) ?? [],
    cloneDraft: (d) => d.map((draft) => ({ ...draft })),
    isDirty: hasSystemDraftChanges,
    isValid: (drafts) =>
      drafts.every(({ level }) => isInRange(level, SYSTEM_LEVEL_MIN, SYSTEM_LEVEL_MAX)),
    applyToGateway: (drafts) =>
      gateway.applySystems({
        systems: drafts.map(({ id, level }) => ({ id, level: level! })),
      }),
  });

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

function hasSystemDraftChanges(drafts: SystemDraft[], lastSeededDrafts: SystemDraft[]) {
  if (drafts.length !== lastSeededDrafts.length) {
    return drafts.length > 0 || lastSeededDrafts.length > 0;
  }

  return drafts.some((draft, index) => {
    const lastSeededDraft = lastSeededDrafts[index];
    return (
      !lastSeededDraft || lastSeededDraft.id !== draft.id || lastSeededDraft.level !== draft.level
    );
  });
}
