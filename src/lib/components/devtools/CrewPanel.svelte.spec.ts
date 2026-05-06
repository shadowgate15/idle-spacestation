import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Component } from 'svelte';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createGameGateway } from '$lib/game/api/gateway';
import CrewPanel from './CrewPanel.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(crewTotal: number) {
  return {
    ...baseSnapshot,
    resources: {
      ...baseSnapshot.resources,
      crew: {
        ...baseSnapshot.resources.crew,
        total: crewTotal,
        available: Math.max(crewTotal - baseSnapshot.resources.crew.assigned, 0),
      },
    },
  };
}

function createGateway() {
  return {
    applyCrew: vi.fn(),
  } as unknown as ReturnType<typeof createGameGateway>;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('CrewPanel', () => {
  it('renders with a null snapshot', async () => {
    const gateway = createGateway();
    const view = await mount(CrewPanel as Component, { props: { snapshot: null, gateway } });

    try {
      await expect.element(page.getByTestId('devtools-crew-panel')).toBeInTheDocument();
      await expect.element(page.getByTestId('devtools-crew-total-input')).toBeDisabled();
      await expect.element(page.getByText('Assigned')).toBeInTheDocument();
      await expect.element(page.getByText('Available')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('renders snapshot values', async () => {
    const gateway = createGateway();
    const snapshot = createSnapshot(9);
    const view = await mount(CrewPanel as Component, { props: { snapshot, gateway } });

    try {
      await expect.element(page.getByTestId('devtools-crew-total-input')).toHaveValue(9);
      await expect.element(page.getByText(String(snapshot.resources.crew.assigned))).toBeInTheDocument();
      await expect.element(page.getByText(String(snapshot.resources.crew.available))).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('applies crew changes and updates displayed values', async () => {
    const gateway = createGateway();
    const responseSnapshot = createSnapshot(14);
    gateway.applyCrew = vi.fn().mockResolvedValue({ ok: true, snapshot: responseSnapshot });

    const view = await mount(CrewPanel as Component, {
      props: { snapshot: createSnapshot(6), gateway },
    });

    try {
      await page.getByTestId('devtools-crew-total-input').fill('14');
      await page.getByTestId('devtools-crew-apply').click();

      expect(gateway.applyCrew).toHaveBeenCalledWith({ crewTotal: 14 });
      await expect.element(page.getByTestId('devtools-crew-total-input')).toHaveValue(14);
      await expect.element(page.getByText(String(responseSnapshot.resources.crew.available))).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('shows an inline error when apply fails', async () => {
    const gateway = createGateway();
    gateway.applyCrew = vi.fn().mockResolvedValue({
      ok: false,
      reasonCode: 'constraint_violation',
      snapshot: createSnapshot(6),
    });

    const view = await mount(CrewPanel as Component, {
      props: { snapshot: createSnapshot(6), gateway },
    });

    try {
      await page.getByTestId('devtools-crew-apply').click();
      await expect.element(page.getByTestId('devtools-crew-error')).toHaveTextContent('constraint_violation');
    } finally {
      await view.unmount();
    }
  });
});
