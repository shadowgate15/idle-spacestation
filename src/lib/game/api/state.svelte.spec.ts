import { beforeEach, describe, expect, it, vi } from 'vitest';

import { adaptGameSnapshot } from './adapters';
import { gameGateway } from './gateway';
import { buildSnapshotFromFixtureState, createFixtureState } from './testing';
import { GameStateInitError, type GameSnapshot, type RawGameSnapshot } from './types';

let gameState: Awaited<typeof import('./state.svelte')>['gameState'];

vi.mock('./gateway', () => ({
  gameGateway: {
    getSnapshot: vi.fn(),
    subscribeToStateChanges: vi.fn(),
  },
}));

describe('gameState', () => {
  beforeEach(async () => {
    vi.clearAllMocks();

    const module = await import('./state.svelte');
    gameState = module.gameState;
    gameState.dispose();
  });

  it('starts in idle status', () => {
    expect(gameState.status).toBe('idle');
    expect(gameState.snapshot).toBeNull();
    expect(gameState.error).toBeNull();
  });

  it('registers listener before calling getSnapshot', async () => {
    const callOrder: string[] = [];
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(() => {
      callOrder.push('subscribe');
      return () => {};
    });
    vi.mocked(gameGateway.getSnapshot).mockImplementation(async () => {
      callOrder.push('getSnapshot');
      return buildTestSnapshot(1);
    });

    await gameState.init();

    expect(callOrder).toEqual(['subscribe', 'getSnapshot']);
  });

  it('sets status to ready and stores the bootstrapped snapshot after init', async () => {
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(() => {});
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(2));

    await gameState.init();

    expect(gameState.status).toBe('ready');
    expect(gameState.snapshot?.meta.tickCount).toBe(2);
    expect(gameState.error).toBeNull();
  });

  it('is idempotent while initialization is pending', async () => {
    let resolveGetSnapshot!: (snapshot: GameSnapshot) => void;
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(() => {});
    vi.mocked(gameGateway.getSnapshot).mockReturnValue(
      new Promise((resolve) => {
        resolveGetSnapshot = resolve;
      }),
    );

    const first = gameState.init();
    const second = gameState.init();
    resolveGetSnapshot(buildTestSnapshot(3));
    await Promise.all([first, second]);

    expect(vi.mocked(gameGateway.subscribeToStateChanges)).toHaveBeenCalledOnce();
    expect(vi.mocked(gameGateway.getSnapshot)).toHaveBeenCalledOnce();
  });

  it('does not reinitialize after status is ready', async () => {
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(() => {});
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(4));

    await gameState.init();
    await gameState.init();

    expect(vi.mocked(gameGateway.subscribeToStateChanges)).toHaveBeenCalledOnce();
    expect(vi.mocked(gameGateway.getSnapshot)).toHaveBeenCalledOnce();
  });

  it('applies push event with higher tickCount', async () => {
    let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any) => {
        pushCallback = callback;
        return () => {};
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(5));

    await gameState.init();
    pushCallback?.(buildRawTestSnapshot(10));

    expect(gameState.snapshot?.meta.tickCount).toBe(10);
    expect(gameState.snapshot?.routes).toBeDefined();
  });

  it('applies push event with equal tickCount', async () => {
    let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any) => {
        pushCallback = callback;
        return () => {};
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(6));

    await gameState.init();
    pushCallback?.(buildRawTestSnapshot(6, 1234));

    expect(gameState.snapshot?.meta.tickCount).toBe(6);
    expect(gameState.snapshot?.resources.materials).toBe(1234);
  });

  it('ignores push event with lower tickCount', async () => {
    let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any) => {
        pushCallback = callback;
        return () => {};
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(10));

    await gameState.init();
    pushCallback?.(buildRawTestSnapshot(3, 4321));

    expect(gameState.snapshot?.meta.tickCount).toBe(10);
    expect(gameState.snapshot?.resources.materials).not.toBe(4321);
  });

  it('keeps newer push event when older bootstrap snapshot resolves later', async () => {
    let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
    let resolveGetSnapshot!: (snapshot: GameSnapshot) => void;
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any) => {
        pushCallback = callback;
        return () => {};
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockReturnValue(
      new Promise((resolve) => {
        resolveGetSnapshot = resolve;
      }),
    );

    const initPromise = gameState.init();
    pushCallback?.(buildRawTestSnapshot(8));
    resolveGetSnapshot(buildTestSnapshot(5));
    await initPromise;

    expect(gameState.snapshot?.meta.tickCount).toBe(8);
    expect(gameState.status).toBe('ready');
  });

  it('uses bootstrap snapshot when it is newer than a push event during init', async () => {
    let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
    let resolveGetSnapshot!: (snapshot: GameSnapshot) => void;
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any) => {
        pushCallback = callback;
        return () => {};
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockReturnValue(
      new Promise((resolve) => {
        resolveGetSnapshot = resolve;
      }),
    );

    const initPromise = gameState.init();
    pushCallback?.(buildRawTestSnapshot(8));
    resolveGetSnapshot(buildTestSnapshot(9));
    await initPromise;

    expect(gameState.snapshot?.meta.tickCount).toBe(9);
  });

  it('dispose resets state and unsubscribes', async () => {
    const unsubscribe = vi.fn();
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(unsubscribe);
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(11));

    await gameState.init();
    gameState.dispose();

    expect(unsubscribe).toHaveBeenCalledOnce();
    expect(gameState.status).toBe('idle');
    expect(gameState.snapshot).toBeNull();
    expect(gameState.error).toBeNull();
  });

  it('can initialize again after dispose', async () => {
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(() => {});
    vi.mocked(gameGateway.getSnapshot)
      .mockResolvedValueOnce(buildTestSnapshot(12))
      .mockResolvedValueOnce(buildTestSnapshot(13));

    await gameState.init();
    gameState.dispose();
    await gameState.init();

    expect(gameState.status).toBe('ready');
    expect(gameState.snapshot?.meta.tickCount).toBe(13);
    expect(vi.mocked(gameGateway.subscribeToStateChanges)).toHaveBeenCalledTimes(2);
  });

  it('throws GameStateInitError and stores error when listener registration fails', async () => {
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(() => {
      throw new Error('network error');
    });

    await expect(gameState.init()).rejects.toBeInstanceOf(GameStateInitError);
    expect(gameState.status).toBe('error');
    expect(gameState.error?.message).toBe('network error');
  });

  it('enters error state when onError callback is invoked post-init', async () => {
    let onErrorFn: ((err: Error) => void) | undefined;
    const unsubscribeFn = vi.fn();
    vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (callback: any, onError: any) => {
        onErrorFn = onError;
        return unsubscribeFn;
      },
    );
    vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(5));

    await gameState.init();

    expect(gameState.status).toBe('ready');
    expect(gameState.error).toBeNull();

    // Simulate async listener registration failure post-init
    onErrorFn?.(new Error('listener failed'));

    expect(gameState.status).toBe('error');
    expect(gameState.error?.message).toBe('listener failed');
    expect(unsubscribeFn).toHaveBeenCalledOnce();
  });

  it('throws GameStateInitError and unsubscribes when getSnapshot fails', async () => {
    const unsubscribe = vi.fn();
    vi.mocked(gameGateway.subscribeToStateChanges).mockReturnValue(unsubscribe);
    vi.mocked(gameGateway.getSnapshot).mockRejectedValue(new Error('rpc failed'));

    await expect(gameState.init()).rejects.toMatchObject({
      name: 'GameStateInitError',
      message: 'gameState.init failed: rpc failed',
    });

    expect(unsubscribe).toHaveBeenCalledOnce();
    expect(gameState.status).toBe('error');
    expect(gameState.error?.message).toBe('rpc failed');
  });

  describe('deferUntilBlur', () => {
    it('defers push events while focused', async () => {
      const pushCallback = await initializeWithPushCallback(2);

      gameState.deferUntilBlur(true);
      pushCallback(buildRawTestSnapshot(5, 500));

      expect(gameState.snapshot?.meta.tickCount).toBe(2);
      expect(gameState.snapshot?.resources.materials).toBe(2);
    });

    it('applies latest deferred event on blur', async () => {
      const pushCallback = await initializeWithPushCallback(2);

      gameState.deferUntilBlur(true);
      pushCallback(buildRawTestSnapshot(5, 500));
      gameState.deferUntilBlur(false);

      expect(gameState.snapshot?.meta.tickCount).toBe(5);
      expect(gameState.snapshot?.resources.materials).toBe(500);
    });

    it('keeps only latest deferred event', async () => {
      const pushCallback = await initializeWithPushCallback(2);

      gameState.deferUntilBlur(true);
      pushCallback(buildRawTestSnapshot(3, 300));
      pushCallback(buildRawTestSnapshot(7, 700));
      pushCallback(buildRawTestSnapshot(5, 500));
      gameState.deferUntilBlur(false);

      expect(gameState.snapshot?.meta.tickCount).toBe(7);
      expect(gameState.snapshot?.resources.materials).toBe(700);
    });

    it('drops stale deferred events', async () => {
      const pushCallback = await initializeWithPushCallback(10);

      gameState.deferUntilBlur(true);
      pushCallback(buildRawTestSnapshot(3, 300));
      gameState.deferUntilBlur(false);

      expect(gameState.snapshot?.meta.tickCount).toBe(10);
      expect(gameState.snapshot?.resources.materials).toBe(10);
    });

    it('applies push events normally after blur', async () => {
      const pushCallback = await initializeWithPushCallback(2);

      gameState.deferUntilBlur(true);
      gameState.deferUntilBlur(false);
      pushCallback(buildRawTestSnapshot(6, 600));

      expect(gameState.snapshot?.meta.tickCount).toBe(6);
      expect(gameState.snapshot?.resources.materials).toBe(600);
    });
  });
});

async function initializeWithPushCallback(
  initialTickCount: number,
): Promise<(raw: RawGameSnapshot) => void> {
  let pushCallback: ((raw: RawGameSnapshot) => void) | undefined;
  vi.mocked(gameGateway.subscribeToStateChanges).mockImplementation((callback) => {
    pushCallback = callback;
    return () => {};
  });
  vi.mocked(gameGateway.getSnapshot).mockResolvedValue(buildTestSnapshot(initialTickCount));

  await gameState.init();

  if (!pushCallback) {
    throw new Error('state change listener was not registered');
  }

  return pushCallback;
}

function buildTestSnapshot(tickCount: number, materials = tickCount): GameSnapshot {
  return adaptGameSnapshot(buildRawTestSnapshot(tickCount, materials));
}

function buildRawTestSnapshot(tickCount: number, materials = tickCount): RawGameSnapshot {
  const state = createFixtureState('starter');
  state.tickCount = tickCount;
  state.materials = materials;

  return buildSnapshotFromFixtureState(state, 'starter');
}
