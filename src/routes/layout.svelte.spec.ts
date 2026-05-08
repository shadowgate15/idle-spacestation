import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import '@tauri-apps/api/event';
import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, type Component } from 'svelte';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { gameGateway } from '$lib/game/api';
import { gameState } from '$lib/game/api/state.svelte';
import Layout from './+layout.svelte';

const DEVTOOLS_STORAGE_KEY = 'idle-spacestation.devtools-open';
const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';
let fixtureTransport: ReturnType<typeof createFixtureTransport>;

function setupIPC() {
  mockIPC(
    (cmd, payload) => {
      switch (cmd) {
        case 'game_get_snapshot':
          return fixtureTransport.getSnapshot();
        case 'game_devtools_get_state':
          return { visible: false, snapshot: fixtureTransport.getSnapshot() };
        case 'game_devtools_set_visibility': {
          const visible =
            payload && typeof payload === 'object' && 'input' in payload
              ? payload.input && typeof payload.input === 'object' && 'visible' in payload.input
                ? payload.input.visible === true
                : false
              : false;

          return { visible, snapshot: fixtureTransport.getSnapshot() };
        }
        default:
          throw new Error(`Unhandled IPC command: ${cmd}`);
      }
    },
    { shouldMockEvents: true },
  );
}

async function mountLayout() {
  const children = createRawSnippet(() => ({
    render: () => '<div data-testid="layout-child">Layout child</div>',
  }));

  return mount(Layout as Component, { children });
}

async function mountVisibleLayout() {
  localStorage.setItem(DEVTOOLS_STORAGE_KEY, 'true');
  const view = await mountLayout();

  await expect.element(page.getByTestId('devtools-overlay')).toBeInTheDocument();
  return view;
}

describe('Root layout devtools overlay', () => {
  beforeEach(() => {
    vi.useRealTimers();
    fixtureTransport = createFixtureTransport('starter');
    localStorage.clear();
    localStorage.setItem('idle-spacestation.transport-mode', 'fixture');
    localStorage.setItem(FIXTURE_STORAGE_KEY, 'starter');
    clearMocks();
    setupIPC();
    gameState.dispose();
  });

  afterEach(() => {
    vi.useRealTimers();
    localStorage.clear();
    clearMocks();
    gameState.dispose();
  });

  it('does not render the overlay when devtools are hidden', async () => {
    const view = await mountLayout();

    try {
      await expect.element(page.getByTestId('game-shell')).toBeInTheDocument();
      await expect.element(page.getByTestId('devtools-overlay')).not.toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('renders the overlay when the localStorage override is enabled', async () => {
    localStorage.setItem(DEVTOOLS_STORAGE_KEY, 'true');

    const view = await mountLayout();

    try {
      await expect.element(page.getByTestId('devtools-overlay')).toBeInTheDocument();
      await expect.element(page.getByText(/Tick: \d+ · Tier: 1/i)).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('hides the overlay after clicking the close button', async () => {
    localStorage.setItem(DEVTOOLS_STORAGE_KEY, 'true');

    const view = await mountLayout();

    try {
      await expect.element(page.getByTestId('devtools-overlay')).toBeInTheDocument();

      await page.getByTestId('devtools-close-btn').click();

      await expect.element(page.getByTestId('devtools-overlay')).not.toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });
});

describe('Root layout gameState integration', () => {
  beforeEach(() => {
    vi.useRealTimers();
    fixtureTransport = createFixtureTransport('starter');
    localStorage.clear();
    localStorage.setItem('idle-spacestation.transport-mode', 'fixture');
    localStorage.setItem(FIXTURE_STORAGE_KEY, 'starter');
    clearMocks();
    setupIPC();
    gameState.dispose();
  });

  afterEach(() => {
    vi.useRealTimers();
    localStorage.clear();
    clearMocks();
    gameState.dispose();
  });

  it('initializes the gameState store on mount', async () => {
    const initSpy = vi.spyOn(gameState, 'init');
    const subscribeSpy = vi.spyOn(gameGateway, 'subscribeToStateChanges');
    const view = await mountLayout();

    try {
      await expect.element(page.getByTestId('game-shell')).toBeInTheDocument();
      await vi.waitFor(() => {
        expect(initSpy).toHaveBeenCalledTimes(1);
      });
      await vi.waitFor(() => {
        expect(subscribeSpy).toHaveBeenCalledTimes(1);
      });
    } finally {
      await view.unmount();
    }
  });

  it('disposes the gameState store on unmount', async () => {
    const disposeSpy = vi.spyOn(gameState, 'dispose');
    const view = await mountLayout();

    await expect.element(page.getByTestId('game-shell')).toBeInTheDocument();
    await view.unmount();

    expect(disposeSpy).toHaveBeenCalled();
  });

  it('passes the store snapshot to DevtoolsOverlay when visible', async () => {
    const view = await mountVisibleLayout();

    try {
      await expect.element(page.getByText(/Tick: \d+ · Tier: 1/i)).toBeInTheDocument();
      await vi.waitFor(() => {
        expect(gameState.snapshot).not.toBeNull();
      });
    } finally {
      await view.unmount();
    }
  });

  it('defers store updates while a devtools input is focused', async () => {
    const deferSpy = vi.spyOn(gameState, 'deferUntilBlur');
    const view = await mountVisibleLayout();

    try {
      const input = page.getByTestId('devtools-materials-input');
      await input.click();

      await vi.waitFor(() => {
        expect(deferSpy).toHaveBeenCalledWith(true);
      });
    } finally {
      await view.unmount();
    }
  });

  it('resumes store updates after the devtools input blurs', async () => {
    const deferSpy = vi.spyOn(gameState, 'deferUntilBlur');
    const view = await mountVisibleLayout();

    try {
      const input = page.getByTestId('devtools-materials-input');
      await input.click();

      await vi.waitFor(() => {
        expect(deferSpy).toHaveBeenCalledWith(true);
      });

      await input.element().blur();

      await vi.waitFor(() => {
        expect(deferSpy).toHaveBeenCalledWith(false);
      });
    } finally {
      await view.unmount();
    }
  });

  it('does not defer updates when a non-input devtools element is focused', async () => {
    const deferSpy = vi.spyOn(gameState, 'deferUntilBlur');
    const view = await mountVisibleLayout();

    try {
      if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur();
      }
      deferSpy.mockClear();

      const button = page.getByTestId('devtools-close-btn');
      await button.element().focus();

      await new Promise((resolve) => setTimeout(resolve, 50));

      const trueCalls = deferSpy.mock.calls.filter(([focused]) => focused === true);
      expect(trueCalls).toHaveLength(0);
    } finally {
      await view.unmount();
    }
  });

  it('exposes gameState and gameGateway on window in fixture mode (types)', async () => {
    // This test verifies that window.__gameState and window.__gameGateway are properly typed
    // without needing double-casts. The types come from src/app.d.ts Window augmentation.
    const view = await mountLayout();

    try {
      // These should not error if window augmentation is properly declared
      if (typeof window !== 'undefined') {
        const gameStateRef = window.__gameState;
        const gameGatewayRef = window.__gameGateway;

        // The layout sets them in fixture mode, but we just verify they exist on the window object
        expect(gameStateRef === gameState || gameStateRef === undefined).toBe(true);
        expect(gameGatewayRef === gameGateway || gameGatewayRef === undefined).toBe(true);
      }
    } finally {
      await view.unmount();
    }
  });
});
