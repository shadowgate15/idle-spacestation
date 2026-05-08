import type { GameSnapshot } from '$lib/game/api/types';

export type ApplyResponse = {
  ok: boolean;
  reasonCode?: string;
  snapshot: GameSnapshot;
};

export interface ApplyPanelStateOptions<TDraft, TResponse extends ApplyResponse> {
  /** Initial draft value from a snapshot (or defaults if null) */
  seedDraft: (snapshot: GameSnapshot | null) => TDraft;
  /** Clone the draft for baseline comparison */
  cloneDraft: (draft: TDraft) => TDraft;
  /** Check if draft differs from baseline */
  isDirty: (draft: TDraft, baseline: TDraft) => boolean;
  /** Validate draft before applying (return false to set invalid_range error) */
  isValid: (draft: TDraft) => boolean;
  /** Call the gateway with the current draft */
  applyToGateway: (draft: TDraft) => Promise<TResponse>;
  /** When true, also reseed drafts from response.snapshot on failure (default: false) */
  reseedOnFailure?: boolean;
}

export function createApplyPanelState<TDraft, TResponse extends ApplyResponse>(
  initialSnapshot: GameSnapshot | null,
  options: ApplyPanelStateOptions<TDraft, TResponse>,
) {
  const initialDraft = options.seedDraft(initialSnapshot);
  let currentSnapshot = $state<GameSnapshot | null>(initialSnapshot);
  let draft = $state(initialDraft);
  let lastSeededDraft = $state(options.cloneDraft(initialDraft));
  let hasSeededOnce = initialSnapshot !== null;
  let errorMessage = $state<string | null>(null);
  let isApplying = $state(false);

  const isDirty = $derived(options.isDirty(draft, lastSeededDraft));

  function reseedDrafts(snapshot: GameSnapshot) {
    const seed = options.seedDraft(snapshot);
    applySeedToDraft(seed);
    lastSeededDraft = options.cloneDraft(draft);
  }

  function applySeedToDraft(seed: TDraft) {
    if (Array.isArray(draft) && Array.isArray(seed)) {
      const target = draft as unknown[];
      target.splice(0, target.length, ...(seed as unknown[]));
      return;
    }

    if (
      draft !== null &&
      typeof draft === 'object' &&
      seed !== null &&
      typeof seed === 'object'
    ) {
      const target = draft as Record<string, unknown>;
      const source = seed as Record<string, unknown>;
      for (const key of Object.keys(target)) {
        if (!(key in source)) delete target[key];
      }
      Object.assign(target, source);
      return;
    }

    draft = seed;
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
    if (!options.isValid(draft)) {
      errorMessage = 'invalid_range';
      return;
    }

    isApplying = true;
    errorMessage = null;

    try {
      const response = await options.applyToGateway(draft);
      currentSnapshot = response.snapshot;

      if (response.ok) {
        reseedDrafts(response.snapshot);
        errorMessage = null;
        return;
      }

      if (options.reseedOnFailure) {
        reseedDrafts(response.snapshot);
      }
      errorMessage = response.reasonCode ?? null;
    } finally {
      isApplying = false;
    }
  }

  return {
    get snapshot() {
      return currentSnapshot;
    },
    get draft() {
      return draft;
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
