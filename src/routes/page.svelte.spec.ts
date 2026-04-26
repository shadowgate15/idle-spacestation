import { afterEach, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { clearMocks } from '@tauri-apps/api/mocks';
import { render } from 'vitest-browser-svelte';
import Page from './+page.svelte';

afterEach(() => {
  clearMocks();
});

describe('+page.svelte tactical shell', () => {
  it('renders tactical shell heading and header status', async () => {
    // In unit tests the layout shell is not rendered, but the page content is
    // Test that the page section headings and hero are present
    render(Page);
    await expect
      .element(page.getByRole('heading', { level: 2, name: /Station Command/i }))
      .toBeInTheDocument();
  });

  it('renders hero callouts and resource strip', async () => {
    render(Page);
    await expect.element(page.getByTestId('home-hero')).toBeInTheDocument();
    await expect.element(page.getByTestId('resource-strip')).toBeInTheDocument();
    await expect.element(page.getByTestId('primary-action')).toBeInTheDocument();
    await expect.element(page.getByText('Review Station Status')).toBeInTheDocument();
    await expect.element(page.getByText('Power Reserve')).toBeInTheDocument();
    await expect.element(page.getByText('82%')).toBeInTheDocument();
    await expect.element(page.getByText('Oxygen Uptime')).toBeInTheDocument();
    await expect.element(page.getByText('99.2%')).toBeInTheDocument();
    await expect.element(page.getByText('Crew Morale')).toBeInTheDocument();
    await expect.element(page.getByText('Stable')).toBeInTheDocument();
  });

  it('renders overview, systems, and alerts panels', async () => {
    render(Page);
    await expect.element(page.getByTestId('overview-panel')).toBeInTheDocument();
    await expect.element(page.getByTestId('systems-panel')).toBeInTheDocument();
    await expect.element(page.getByTestId('alerts-panel')).toBeInTheDocument();
    await expect.element(page.getByText('Operational Snapshot')).toBeInTheDocument();
    await expect.element(page.getByText('Priority Systems')).toBeInTheDocument();
    await expect.element(page.getByText('Command Alerts')).toBeInTheDocument();
    await expect
      .element(page.getByText('Solar lattice holding steady across the sunline.'))
      .toBeInTheDocument();
    await expect.element(page.getByText('Reactor')).toBeInTheDocument();
    await expect
      .element(page.getByText('Cooling loop efficiency dipped 3% overnight.'))
      .toBeInTheDocument();
  });

  it('does not render template greet controls or logos', async () => {
    render(Page);
    await expect.element(page.getByRole('button', { name: /greet/i })).not.toBeInTheDocument();
    await expect.element(page.getByPlaceholder(/Enter a name\.{3}/)).not.toBeInTheDocument();
  });
});
