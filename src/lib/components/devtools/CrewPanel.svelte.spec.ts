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

  it('polling does not overwrite typed crew draft', async () => {
    const gateway = createGateway();
    const initialSnapshot = createSnapshot(6);
    const view = await mount(CrewPanel as Component, {
      props: { snapshot: initialSnapshot, gateway },
    });

    try {
      const crewInput = page.getByTestId('devtools-crew-total-input');
      await crewInput.fill('5');
      await expect.element(crewInput).toHaveValue(5);

      const crewEl = crewInput.element() as HTMLInputElement;
      crewEl.focus();
      expect(document.activeElement).toBe(crewEl);

      await view.rerender({ snapshot: createSnapshot(7), gateway });

      await expect.element(crewInput).toHaveValue(5);
      expect(document.activeElement).toBe(crewEl);
    } finally {
      await view.unmount();
    }
  });

  it('Apply success reseeds crew draft to response.snapshot', async () => {
    const gateway = createGateway();
    const responseSnapshot = createSnapshot(20);
    gateway.applyCrew = vi.fn().mockResolvedValue({ ok: true, snapshot: responseSnapshot });

    const view = await mount(CrewPanel as Component, {
      props: { snapshot: createSnapshot(6), gateway },
    });

    try {
      await page.getByTestId('devtools-crew-total-input').fill('20');
      await page.getByTestId('devtools-crew-apply').click();

      await expect.element(page.getByTestId('devtools-crew-total-input')).toHaveValue(20);

      await view.rerender({ snapshot: responseSnapshot, gateway });
      await expect.element(page.getByTestId('devtools-crew-total-input')).toHaveValue(20);
      await expect.element(page.getByTestId('devtools-crew-error')).toHaveTextContent('');
    } finally {
      await view.unmount();
    }
  });

  it('Apply failure reseeds crew draft to response.snapshot', async () => {
    const gateway = createGateway();
    const failureSnapshot = createSnapshot(6);
    gateway.applyCrew = vi.fn().mockResolvedValue({
      ok: false,
      reasonCode: 'constraint_violation',
      snapshot: failureSnapshot,
    });

    const view = await mount(CrewPanel as Component, {
      props: { snapshot: createSnapshot(6), gateway },
    });

    try {
      await page.getByTestId('devtools-crew-total-input').fill('99');
      await page.getByTestId('devtools-crew-apply').click();

      await expect.element(page.getByTestId('devtools-crew-total-input')).toHaveValue(6);
      await expect.element(page.getByTestId('devtools-crew-error')).toHaveTextContent('constraint_violation');
    } finally {
      await view.unmount();
    }
  });
});
