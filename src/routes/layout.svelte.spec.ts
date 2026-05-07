import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import '@tauri-apps/api/event';
import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, type Component } from 'svelte';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import Layout from './+layout.svelte';

const DEVTOOLS_STORAGE_KEY = 'idle-spacestation.devtools-open';
const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';
let fixtureTransport: ReturnType<typeof createFixtureTransport>;

function setupIPC() {
  mockIPC(
    (cmd, payload) => {
      switch (cmd) {
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

async function advanceOnePollingInterval() {
  await vi.advanceTimersByTimeAsync(1100);
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
  });

  afterEach(() => {
    vi.useRealTimers();
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

  it('blocks polling while a devtools input is focused', async () => {
    vi.useFakeTimers();
    const getSnapshotSpy = vi.spyOn(fixtureTransport, 'getSnapshot');
    const view = await mountVisibleLayout();

    try {
      const input = page.getByTestId('devtools-materials-input');
      await input.click();
      const baselineCallCount = getSnapshotSpy.mock.calls.length;

      await advanceOnePollingInterval();

      expect(getSnapshotSpy).toHaveBeenCalledTimes(baselineCallCount);
    } finally {
      await view.unmount();
    }
  });

  it('resumes polling after a focused devtools input blurs', async () => {
    vi.useFakeTimers();
    const getSnapshotSpy = vi.spyOn(fixtureTransport, 'getSnapshot');
    const view = await mountVisibleLayout();

    try {
      const input = page.getByTestId('devtools-materials-input');
      await input.click();
      const focusedCallCount = getSnapshotSpy.mock.calls.length;

      await advanceOnePollingInterval();
      expect(getSnapshotSpy).toHaveBeenCalledTimes(focusedCallCount);

      await input.element().blur();
      await advanceOnePollingInterval();

      expect(getSnapshotSpy).toHaveBeenCalledTimes(focusedCallCount + 1);
    } finally {
      await view.unmount();
    }
  });

  it('self-heals polling when the focused devtools input is removed', async () => {
    vi.useFakeTimers();
    const getSnapshotSpy = vi.spyOn(fixtureTransport, 'getSnapshot');
    const view = await mountVisibleLayout();

    try {
      const input = page.getByTestId('devtools-materials-input');
      await input.click();
      const focusedCallCount = getSnapshotSpy.mock.calls.length;

      await advanceOnePollingInterval();
      expect(getSnapshotSpy).toHaveBeenCalledTimes(focusedCallCount);

      input.element().remove();
      await advanceOnePollingInterval();

      expect(getSnapshotSpy).toHaveBeenCalledTimes(focusedCallCount + 1);
    } finally {
      await view.unmount();
    }
  });

  it('continues polling when a devtools button is focused', async () => {
    vi.useFakeTimers();
    const getSnapshotSpy = vi.spyOn(fixtureTransport, 'getSnapshot');
    const view = await mountVisibleLayout();

    try {
      const button = page.getByTestId('devtools-close-btn');
      await button.element().focus();
      const baselineCallCount = getSnapshotSpy.mock.calls.length;

      await advanceOnePollingInterval();

      expect(getSnapshotSpy).toHaveBeenCalledTimes(baselineCallCount + 1);
    } finally {
      await view.unmount();
    }
  });
});
