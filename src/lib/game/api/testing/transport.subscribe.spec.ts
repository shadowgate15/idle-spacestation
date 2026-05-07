import { describe, expect, it, vi } from 'vitest';
import { createFixtureTransport } from './transport';
import type { RawGameSnapshot } from '../types';

describe('fixtureTransport.subscribeToStateChanges', () => {
  it('fires callback after mutation', async () => {
    const transport = createFixtureTransport('starter');
    const received: RawGameSnapshot[] = [];
    transport.subscribeToStateChanges((raw) => received.push(raw));
    await transport.invoke('game_devtools_reset_to_starter', {});
    expect(received).toHaveLength(1);
  });

  it('fires all callbacks when multiple subscribers', async () => {
    const transport = createFixtureTransport('starter');
    const cb1 = vi.fn();
    const cb2 = vi.fn();
    transport.subscribeToStateChanges(cb1);
    transport.subscribeToStateChanges(cb2);
    await transport.invoke('game_devtools_reset_to_starter', {});
    expect(cb1).toHaveBeenCalledOnce();
    expect(cb2).toHaveBeenCalledOnce();
  });

  it('does not fire after unsubscribe', async () => {
    const transport = createFixtureTransport('starter');
    const cb = vi.fn();
    const unsubscribe = transport.subscribeToStateChanges(cb);
    unsubscribe();
    await transport.invoke('game_devtools_reset_to_starter', {});
    expect(cb).not.toHaveBeenCalled();
  });

  it('does not fire on read-only commands', async () => {
    const transport = createFixtureTransport('starter');
    const cb = vi.fn();
    transport.subscribeToStateChanges(cb);
    await transport.invoke('game_get_snapshot', undefined);
    await transport.invoke('game_devtools_get_state', undefined);
    expect(cb).not.toHaveBeenCalled();
  });

  it('fires only once for advance_ticks(N)', async () => {
    const transport = createFixtureTransport('starter');
    const cb = vi.fn();
    transport.subscribeToStateChanges(cb);
    await transport.invoke('game_devtools_advance_ticks', { count: 5 });
    expect(cb).toHaveBeenCalledOnce();
  });

  it('two transports have independent subscriber lists', async () => {
    const transportA = createFixtureTransport('starter');
    const transportB = createFixtureTransport('starter');
    const cbA = vi.fn();
    const cbB = vi.fn();
    transportA.subscribeToStateChanges(cbA);
    transportB.subscribeToStateChanges(cbB);
    await transportA.invoke('game_devtools_reset_to_starter', {});
    expect(cbA).toHaveBeenCalledOnce();
    expect(cbB).not.toHaveBeenCalled();
  });
});
