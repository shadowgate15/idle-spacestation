import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import '@tauri-apps/api/event';
import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createRawSnippet, type Component } from 'svelte';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import Layout from './+layout.svelte';

const DEVTOOLS_STORAGE_KEY = 'idle-spacestation.devtools-open';
const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';
const snapshot = createFixtureTransport('starter').getSnapshot();

function setupIPC() {
  mockIPC(
    (cmd, payload) => {
      switch (cmd) {
        case 'game_devtools_get_state':
          return { visible: false, snapshot };
        case 'game_devtools_set_visibility': {
          const visible =
            payload && typeof payload === 'object' && 'input' in payload
              ? payload.input && typeof payload.input === 'object' && 'visible' in payload.input
                ? payload.input.visible === true
                : false
              : false;

          return { visible, snapshot };
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

describe('Root layout devtools overlay', () => {
  beforeEach(() => {
    localStorage.clear();
    localStorage.setItem('idle-spacestation.transport-mode', 'fixture');
    localStorage.setItem(FIXTURE_STORAGE_KEY, 'starter');
    clearMocks();
    setupIPC();
  });

  afterEach(() => {
    localStorage.clear();
    clearMocks();
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
      await expect.element(page.getByText(/Tick: 0 · Tier: 1/i)).toBeInTheDocument();
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
