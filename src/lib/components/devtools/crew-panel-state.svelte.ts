import type { GameSnapshot, GatewayDevtoolsApplyCrewResponse } from '$lib/game/api/types';
import { isInRange } from '$lib/utils';
import { createApplyPanelState } from './_create-apply-panel-state.svelte';

type CrewGateway = {
  applyCrew: (input: { crewTotal: number }) => Promise<GatewayDevtoolsApplyCrewResponse>;
};

type CrewDraft = {
  crewTotal: number | undefined;
};

const CREW_MIN = 0;
const CREW_MAX = 999;

export function createCrewPanelState(snapshot: GameSnapshot | null, gateway: CrewGateway) {
  const base = createApplyPanelState<CrewDraft, GatewayDevtoolsApplyCrewResponse>(snapshot, {
    seedDraft: (s) => ({ crewTotal: s?.resources.crew.total ?? 0 }),
    cloneDraft: (d) => ({ ...d }),
    isDirty: (d, b) => d.crewTotal !== b.crewTotal,
    isValid: (d) => isInRange(d.crewTotal, CREW_MIN, CREW_MAX),
    applyToGateway: (d) => gateway.applyCrew({ crewTotal: d.crewTotal! }),
    reseedOnFailure: true,
  });

  return {
    get snapshot() {
      return base.snapshot;
    },
    get crewTotalDraft() {
      return base.draft.crewTotal;
    },
    set crewTotalDraft(value: number | undefined) {
      base.draft.crewTotal = value;
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
