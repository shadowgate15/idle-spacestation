import type { GameSnapshot, GatewayDevtoolsApplyResourcesResponse } from '$lib/game/api/types';
import { isInRange } from '$lib/utils';
import { createApplyPanelState } from './_create-apply-panel-state.svelte';

type ResourcesGateway = {
  applyResources: (input: {
    materials: number;
    data: number;
  }) => Promise<GatewayDevtoolsApplyResourcesResponse>;
};

type ResourcesDraft = {
  materials: number | undefined;
  data: number | undefined;
};

const MATERIALS_MIN = 0;
const MATERIALS_MAX = 99999;
const DATA_MIN = 0;
const DATA_MAX = 99999;

export function createResourcesPanelState(
  snapshot: GameSnapshot | null,
  gateway: ResourcesGateway,
) {
  const base = createApplyPanelState<ResourcesDraft, GatewayDevtoolsApplyResourcesResponse>(
    snapshot,
    {
      seedDraft: (s) => ({
        materials: s?.resources.materials ?? 0,
        data: s?.resources.data ?? 0,
      }),
      cloneDraft: (d) => ({ ...d }),
      isDirty: (d, b) => d.materials !== b.materials || d.data !== b.data,
      isValid: (d) =>
        isInRange(d.materials, MATERIALS_MIN, MATERIALS_MAX) &&
        isInRange(d.data, DATA_MIN, DATA_MAX),
      applyToGateway: (d) =>
        gateway.applyResources({ materials: d.materials!, data: d.data! }),
      reseedOnFailure: true,
    },
  );

  return {
    get snapshot() {
      return base.snapshot;
    },
    get materialsDraft() {
      return base.draft.materials;
    },
    set materialsDraft(value: number | undefined) {
      base.draft.materials = value;
    },
    get dataDraft() {
      return base.draft.data;
    },
    set dataDraft(value: number | undefined) {
      base.draft.data = value;
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
