<script lang="ts">
  import type { GameSnapshot } from '$lib/game/api/types';
  import { useSnapshotSync } from './_use-snapshot-sync.svelte';

  let {
    state,
    snapshot,
  }: {
    state: { sync: (s: GameSnapshot | null) => void };
    snapshot: GameSnapshot | null;
  } = $props();

  const adapter = { sync: (s: GameSnapshot | null) => state.sync(s) };
  useSnapshotSync(adapter, () => snapshot);
</script>

<div data-testid="use-snapshot-sync-harness"></div>
