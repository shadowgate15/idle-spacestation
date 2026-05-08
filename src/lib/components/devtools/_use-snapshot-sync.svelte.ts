import { untrack } from 'svelte';
import type { GameSnapshot } from '$lib/game/api/types';

/**
 * Wire a devtools panel state's `sync(snapshot)` to a reactive snapshot getter.
 *
 * The `untrack` call is CRITICAL: without it, any reactive reads INSIDE
 * `state.sync()` (e.g. reads of the panel's own `$state` drafts) would create
 * new tracking dependencies for this `$effect` and cause it to re-run every
 * time those internal values change — producing an infinite loop.
 *
 * Only the `getSnapshot()` call should drive re-runs; everything inside
 * `state.sync` is intentionally untracked.
 *
 * Usage (inside a panel's `<script>` block):
 * ```ts
 *   useSnapshotSync(state, () => snapshot);
 * ```
 *
 * Must be called during component initialisation so it owns the parent
 * component's `$effect` lifecycle. Cleanup is automatic on unmount — there
 * is intentionally no return value.
 */
export function useSnapshotSync(
  state: { sync: (s: GameSnapshot | null) => void },
  getSnapshot: () => GameSnapshot | null,
): void {
  $effect(() => {
    const s = getSnapshot(); // reactive read: re-runs when snapshot changes
    untrack(() => state.sync(s)); // non-reactive: internal reads inside sync don't re-trigger
  });
}
