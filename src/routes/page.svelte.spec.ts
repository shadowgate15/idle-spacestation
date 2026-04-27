import { describe, expect, it, vi, beforeEach } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import { createFixtureTransport } from '$lib/game/api/testing/transport';
import { createGameGateway } from '$lib/game/api/gateway';
import type { PreviewFixtureName } from '$lib/game/api/types';
import type { Component } from 'svelte';
import Page from './+page.svelte';

const FIXTURE_STORAGE_KEY = 'idle-spacestation.e2e-fixture';

function setupFixtureTransport(fixtureName: PreviewFixtureName) {
  const transport = createFixtureTransport(fixtureName);
  localStorage.setItem(FIXTURE_STORAGE_KEY, fixtureName);
  return transport;
}

function clearFixture() {
  localStorage.removeItem(FIXTURE_STORAGE_KEY);
}

describe('Overview page', () => {
  beforeEach(() => {
    clearFixture();
    vi.restoreAllMocks();
  });

  it('renders loading state initially', async () => {
    render(Page as Component);
    await expect.element(page.getByText(/Loading station data/i)).toBeInTheDocument();
  });

  it('renders starter fixture data correctly', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.overview.activePlanet.name).toBe('Solstice Anchor');
    expect(snapshot.routes.overview.stationTier.current).toBe(1);
    expect(snapshot.routes.overview.serviceUtilization.capacity).toBe(2);
    expect(snapshot.routes.overview.surveyProgress.current).toBe(0);
  });

  it('renders deficit fixture with deficit warnings', async () => {
    const transport = setupFixtureTransport('deficit');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const hasDeficitWarnings = snapshot.routes.overview.deficitWarnings;
    expect(hasDeficitWarnings.length).toBeGreaterThan(0);
    expect(hasDeficitWarnings.some((w) => w.code === 'power-deficit')).toBe(true);
  });

  it('renders prestige-ready fixture data correctly', async () => {
    const transport = setupFixtureTransport('prestige-ready');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    expect(snapshot.routes.overview.activePlanet.name).toBe('Aurora Pier');
    expect(snapshot.routes.overview.stationTier.current).toBe(4);
    expect(snapshot.routes.overview.surveyProgress.current).toBe(1550);
  });

  it('displays resource deltas with correct format', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const resourceDeltas = snapshot.routes.overview.resourceDeltas;
    expect(resourceDeltas.length).toBe(4);

    const powerDelta = resourceDeltas.find((r) => r.id === 'power');
    expect(powerDelta).toBeDefined();
    expect(powerDelta!.deltaPerSecond).toBeGreaterThan(0);
  });

  it('displays service utilization summary', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const utilization = snapshot.routes.overview.serviceUtilization;
    expect(utilization.active).toBe(1);
    expect(utilization.capacity).toBe(2);
    expect(utilization.available).toBe(1);
  });

  it('displays planet modifiers', async () => {
    const transport = setupFixtureTransport('starter');
    const gateway = createGameGateway(transport);
    const snapshot = await gateway.getSnapshot();

    const modifiers = snapshot.routes.overview.activePlanet.modifiers;
    expect(modifiers.length).toBe(2);
    expect(modifiers.some((m) => m.target === 'crew-efficiency')).toBe(true);
    expect(modifiers.some((m) => m.target === 'data-output')).toBe(true);
  });
});
