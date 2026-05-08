import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import '@tauri-apps/api/event';
import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createRawSnippet, type Component } from 'svelte';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { gameState } from '$lib/game/api/state.svelte';
import SnapshotGuard from './SnapshotGuard.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function readyChildren() {
  return createRawSnippet(() => ({
    render: () => '<div data-testid="ready-child">ready content</div>',
  }));
}

beforeEach(() => {
  clearMocks();
  gameState.dispose();
});

afterEach(() => {
  clearMocks();
  gameState.dispose();
});

describe('SnapshotGuard', () => {
  it('renders the loading message when status is not ready', async () => {
    const view = await mount(SnapshotGuard as Component, {
      props: {
        loadingMessage: 'Loading station data...',
        children: readyChildren(),
      },
    });

    try {
      await expect.element(page.getByText('Loading station data...')).toBeInTheDocument();
      await expect.element(page.getByTestId('ready-child')).not.toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('uses the default loading message when none is provided', async () => {
    const view = await mount(SnapshotGuard as Component, {
      props: {
        children: readyChildren(),
      },
    });

    try {
      await expect.element(page.getByText('Loading...')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('still renders the loading branch when init has failed (loading takes precedence over error)', async () => {
    mockIPC(
      (cmd) => {
        if (cmd === 'game_get_snapshot') {
          throw new Error('boom: backend offline');
        }
        throw new Error(`Unhandled IPC command: ${cmd}`);
      },
      { shouldMockEvents: true },
    );

    await expect(gameState.init()).rejects.toBeTruthy();
    expect(gameState.status).toBe('error');
    expect(gameState.error?.message).toContain('boom: backend offline');

    const view = await mount(SnapshotGuard as Component, {
      props: {
        loadingMessage: 'Loading station data...',
        children: readyChildren(),
      },
    });

    try {
      await expect.element(page.getByText('Loading station data...')).toBeInTheDocument();
      await expect.element(page.getByTestId('ready-child')).not.toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('renders the children snippet with a non-null snapshot when ready', async () => {
    mockIPC(
      (cmd) => {
        if (cmd === 'game_get_snapshot') {
          return createFixtureTransport('starter').getSnapshot();
        }
        throw new Error(`Unhandled IPC command: ${cmd}`);
      },
      { shouldMockEvents: true },
    );

    await gameState.init();
    expect(gameState.status).toBe('ready');
    expect(gameState.snapshot).not.toBeNull();

    const view = await mount(SnapshotGuard as Component, {
      props: {
        children: readyChildren(),
      },
    });

    try {
      await expect.element(page.getByTestId('ready-child')).toBeInTheDocument();
      await expect.element(page.getByText('Loading...')).not.toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('renders the error block when status is ready and gameState.error is set', async () => {
    mockIPC(
      (cmd) => {
        if (cmd === 'game_get_snapshot') {
          return createFixtureTransport('starter').getSnapshot();
        }
        throw new Error(`Unhandled IPC command: ${cmd}`);
      },
      { shouldMockEvents: true },
    );
    await gameState.init();

    const originalError = Object.getOwnPropertyDescriptor(gameState, 'error');
    Object.defineProperty(gameState, 'error', {
      configurable: true,
      get: () => new Error('boom: backend offline'),
    });

    try {
      const view = await mount(SnapshotGuard as Component, {
        props: {
          children: readyChildren(),
        },
      });

      try {
        await expect.element(page.getByText(/boom: backend offline/i)).toBeInTheDocument();
        await expect.element(page.getByTestId('ready-child')).not.toBeInTheDocument();
      } finally {
        await view.unmount();
      }
    } finally {
      if (originalError) {
        Object.defineProperty(gameState, 'error', originalError);
      }
    }
  });

  it('passes the live GameSnapshot into the children snippet', async () => {
    gameState.applySnapshot(baseSnapshot);

    mockIPC(
      (cmd) => {
        if (cmd === 'game_get_snapshot') {
          return createFixtureTransport('starter').getSnapshot();
        }
        throw new Error(`Unhandled IPC command: ${cmd}`);
      },
      { shouldMockEvents: true },
    );
    await gameState.init();

    const planetName = baseSnapshot.routes.overview.activePlanet.name;
    const tickCount = String(gameState.snapshot!.meta.tickCount);

    const children = createRawSnippet(
      (
        snapshot: () => {
          routes: { overview: { activePlanet: { name: string } } };
          meta: { tickCount: number };
        },
      ) => ({
        render: () => {
          const snap = snapshot();
          return `<div data-testid="snapshot-readout">${snap.routes.overview.activePlanet.name}|${snap.meta.tickCount}</div>`;
        },
      }),
    );

    const view = await mount(SnapshotGuard as Component, {
      props: { children },
    });

    try {
      await expect
        .element(page.getByTestId('snapshot-readout'))
        .toHaveTextContent(`${planetName}|${tickCount}`);
    } finally {
      await view.unmount();
    }
  });
});
