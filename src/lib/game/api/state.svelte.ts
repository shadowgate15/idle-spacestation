import { adaptGameSnapshot } from './adapters';
import { gameGateway } from './gateway';
import type { GameSnapshot, RawGameSnapshot } from './types';
import { GameStateInitError } from './types';

export type GameStateStatus = 'idle' | 'loading' | 'ready' | 'error';

function createGameState() {
  let snapshot = $state<GameSnapshot | null>(null);
  let status = $state<GameStateStatus>('idle');
  let error = $state<Error | null>(null);
  let unlisten: (() => void) | null = null;
  let initPromise: Promise<void> | null = null;
  let isDeferred = false;
  let pendingRaw: RawGameSnapshot | null = null;

  function applyIfNewer(raw: RawGameSnapshot): void {
    const incomingTickCount = raw.meta.tickCount;

    if (snapshot && incomingTickCount < snapshot.meta.tickCount) return;

    if (isDeferred) {
      if (!pendingRaw || incomingTickCount >= pendingRaw.meta.tickCount) {
        pendingRaw = raw;
      }
      return;
    }

    snapshot = adaptGameSnapshot(raw);
  }

  async function init(): Promise<void> {
    if (initPromise) return initPromise;
    if (status === 'ready') return;

    status = 'loading';
    error = null;

    initPromise = (async () => {
      try {
        unlisten = gameGateway.subscribeToStateChanges((raw) => {
          applyIfNewer(raw);
        });

        const bootstrapped = await gameGateway.getSnapshot();

        if (!snapshot || bootstrapped.meta.tickCount >= snapshot.meta.tickCount) {
          snapshot = bootstrapped;
        }

        status = 'ready';
      } catch (err) {
        status = 'error';
        error = err instanceof Error ? err : new Error(String(err));
        unlisten?.();
        unlisten = null;

        throw new GameStateInitError(`gameState.init failed: ${error.message}`, {
          cause: error,
        });
      } finally {
        initPromise = null;
      }
    })();

    return initPromise;
  }

  return {
    get snapshot() {
      return snapshot;
    },
    get status() {
      return status;
    },
    get error() {
      return error;
    },
    init,
    dispose(): void {
      unlisten?.();
      unlisten = null;
      snapshot = null;
      status = 'idle';
      error = null;
      initPromise = null;
      isDeferred = false;
      pendingRaw = null;
    },
    deferUntilBlur(focused: boolean): void {
      if (focused) {
        isDeferred = true;
        return;
      }

      isDeferred = false;

      if (pendingRaw) {
        const raw = pendingRaw;
        pendingRaw = null;
        applyIfNewer(raw);
      }
    },
    _setSnapshot(s: GameSnapshot | null): void {
      snapshot = s;
    },
  };
}

export const gameState = createGameState();
