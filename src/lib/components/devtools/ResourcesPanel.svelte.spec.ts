import { page } from 'vitest/browser';
import { render as mount } from 'vitest-browser-svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Component } from 'svelte';
import { adaptGameSnapshot } from '$lib/game/api';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createGameGateway } from '$lib/game/api/gateway';
import ResourcesPanel from './ResourcesPanel.svelte';

const baseSnapshot = adaptGameSnapshot(createFixtureTransport('starter').getSnapshot());

function createSnapshot(overrides?: Partial<typeof baseSnapshot.resources>) {
  return {
    ...baseSnapshot,
    resources: {
      ...baseSnapshot.resources,
      ...overrides,
      power: overrides?.power ?? baseSnapshot.resources.power,
      crew: overrides?.crew ?? baseSnapshot.resources.crew,
    },
  };
}

function createGateway() {
  return {
    applyResources: vi.fn(),
  } as unknown as ReturnType<typeof createGameGateway>;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('ResourcesPanel', () => {
  it('renders with a null snapshot', async () => {
    const gateway = createGateway();
    const view = await mount(ResourcesPanel as Component, { props: { snapshot: null, gateway } });

    try {
      await expect.element(page.getByTestId('devtools-resources-panel')).toBeInTheDocument();
      await expect.element(page.getByTestId('devtools-materials-input')).toBeDisabled();
      await expect.element(page.getByTestId('devtools-data-input')).toBeDisabled();
      await expect.element(page.getByText('offline')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('renders snapshot values', async () => {
    const gateway = createGateway();
    const snapshot = createSnapshot({ materials: 42, data: 17 });
    const view = await mount(ResourcesPanel as Component, { props: { snapshot, gateway } });

    try {
      await expect.element(page.getByTestId('devtools-materials-input')).toHaveValue(42);
      await expect.element(page.getByTestId('devtools-data-input')).toHaveValue(17);
      await expect
        .element(page.getByText(`${snapshot.resources.power.available} avail`))
        .toBeInTheDocument();
      await expect.element(page.getByText('Power (read only)')).toBeInTheDocument();
    } finally {
      await view.unmount();
    }
  });

  it('applies resource changes and updates displayed values', async () => {
    const gateway = createGateway();
    const responseSnapshot = createSnapshot({ materials: 333, data: 444 });
    gateway.applyResources = vi.fn().mockResolvedValue({ ok: true, snapshot: responseSnapshot });

    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: createSnapshot({ materials: 1, data: 2 }), gateway },
    });

    try {
      await page.getByTestId('devtools-materials-input').fill('333');
      await page.getByTestId('devtools-data-input').fill('444');
      await page.getByTestId('devtools-resources-apply').click();

      expect(gateway.applyResources).toHaveBeenCalledWith({ materials: 333, data: 444 });
      await expect.element(page.getByTestId('devtools-materials-input')).toHaveValue(333);
      await expect.element(page.getByTestId('devtools-data-input')).toHaveValue(444);
    } finally {
      await view.unmount();
    }
  });

  it('shows an inline error when apply fails', async () => {
    const gateway = createGateway();
    gateway.applyResources = vi.fn().mockResolvedValue({
      ok: false,
      reasonCode: 'invalid_state',
      snapshot: createSnapshot({ materials: 12, data: 18 }),
    });

    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: createSnapshot({ materials: 1, data: 2 }), gateway },
    });

    try {
      await page.getByTestId('devtools-resources-apply').click();
      await expect
        .element(page.getByTestId('devtools-resources-error'))
        .toHaveTextContent('invalid_state');
    } finally {
      await view.unmount();
    }
  });

  it('polling does not overwrite typed materials draft', async () => {
    const gateway = createGateway();
    const initialSnapshot = createSnapshot({ materials: 100, data: 200 });
    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: initialSnapshot, gateway },
    });

    try {
      const materialsInput = page.getByTestId('devtools-materials-input');
      await materialsInput.fill('555');
      await expect.element(materialsInput).toHaveValue(555);

      const materialsEl = materialsInput.element() as HTMLInputElement;
      materialsEl.focus();
      expect(document.activeElement).toBe(materialsEl);

      await view.rerender({ snapshot: createSnapshot({ materials: 101, data: 201 }), gateway });

      await expect.element(materialsInput).toHaveValue(555);
      expect(document.activeElement).toBe(materialsEl);
    } finally {
      await view.unmount();
    }
  });

  it('polling does not overwrite typed data draft', async () => {
    const gateway = createGateway();
    const initialSnapshot = createSnapshot({ materials: 100, data: 200 });
    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: initialSnapshot, gateway },
    });

    try {
      const dataInput = page.getByTestId('devtools-data-input');
      await dataInput.fill('777');
      await expect.element(dataInput).toHaveValue(777);

      const dataEl = dataInput.element() as HTMLInputElement;
      dataEl.focus();
      expect(document.activeElement).toBe(dataEl);

      await view.rerender({ snapshot: createSnapshot({ materials: 102, data: 202 }), gateway });

      await expect.element(dataInput).toHaveValue(777);
      expect(document.activeElement).toBe(dataEl);
    } finally {
      await view.unmount();
    }
  });

  it('Apply success reseeds drafts and clears isDirty', async () => {
    const gateway = createGateway();
    const responseSnapshot = createSnapshot({ materials: 333, data: 444 });
    gateway.applyResources = vi.fn().mockResolvedValue({ ok: true, snapshot: responseSnapshot });

    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: createSnapshot({ materials: 1, data: 2 }), gateway },
    });

    try {
      await page.getByTestId('devtools-materials-input').fill('333');
      await page.getByTestId('devtools-data-input').fill('444');
      await page.getByTestId('devtools-resources-apply').click();

      await expect.element(page.getByTestId('devtools-materials-input')).toHaveValue(333);
      await expect.element(page.getByTestId('devtools-data-input')).toHaveValue(444);

      await view.rerender({ snapshot: responseSnapshot, gateway });
      await expect.element(page.getByTestId('devtools-materials-input')).toHaveValue(333);
      await expect.element(page.getByTestId('devtools-data-input')).toHaveValue(444);
      await expect.element(page.getByTestId('devtools-resources-error')).toHaveTextContent('');
    } finally {
      await view.unmount();
    }
  });

  it('Apply failure reseeds drafts to response.snapshot', async () => {
    const gateway = createGateway();
    const failureSnapshot = createSnapshot({ materials: 120, data: 50 });
    gateway.applyResources = vi.fn().mockResolvedValue({
      ok: false,
      reasonCode: 'invalid_range',
      snapshot: failureSnapshot,
    });

    const view = await mount(ResourcesPanel as Component, {
      props: { snapshot: createSnapshot({ materials: 120, data: 50 }), gateway },
    });

    try {
      await page.getByTestId('devtools-materials-input').fill('999');
      await page.getByTestId('devtools-resources-apply').click();

      await expect.element(page.getByTestId('devtools-materials-input')).toHaveValue(120);
      await expect.element(page.getByTestId('devtools-data-input')).toHaveValue(50);
      await expect
        .element(page.getByTestId('devtools-resources-error'))
        .toHaveTextContent('invalid_range');
    } finally {
      await view.unmount();
    }
  });
});
