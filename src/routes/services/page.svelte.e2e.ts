import { expect, test } from '@playwright/test';

test.describe('Services page with live data', () => {
  test('renders all 6 services for starter fixture', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByTestId('services-header')).toBeVisible();
    await expect(page.getByText(/Station Services/)).toBeVisible();

    await expect(page.getByText('Solar Harvester')).toBeVisible();
    await expect(page.getByText('Ore Reclaimer')).toBeVisible();
    await expect(page.getByText('Survey Uplink')).toBeVisible();
    await expect(page.getByText('Maintenance Bay')).toBeVisible();
    await expect(page.getByText('Command Relay')).toBeVisible();
    await expect(page.getByText('Fabrication Loop')).toBeVisible();
  });

  test('shows service utilization', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByText('Active')).toBeVisible();
    await expect(page.getByText('1')).toBeVisible();
    await expect(page.getByText('Capacity')).toBeVisible();
    await expect(page.getByText('2')).toBeVisible();
  });

  test('shows service families', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByText('Production')).toBeVisible();
    await expect(page.getByText('Support')).toBeVisible();
    await expect(page.getByText('Command')).toBeVisible();
    await expect(page.getByText('Conversion')).toBeVisible();
  });

  test('shows status labels for services', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    const solarHarvester = page.locator('card').filter({ hasText: 'Solar Harvester' });
    await expect(solarHarvester.getByText('Active')).toBeVisible();

    const oreReclaimer = page.locator('card').filter({ hasText: 'Ore Reclaimer' });
    await expect(oreReclaimer.getByText('Disabled')).toBeVisible();
  });

  test('shows crew assignment', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    const solarHarvester = page.locator('card').filter({ hasText: 'Solar Harvester' });
    await expect(solarHarvester.getByText(/2 \/ 2/)).toBeVisible();
  });

  test('shows power usage', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    const solarHarvester = page.locator('card').filter({ hasText: 'Solar Harvester' });
    await expect(solarHarvester.getByText(/Power Upkeep/)).toBeVisible();
    await expect(solarHarvester.getByText(/0 \/s/)).toBeVisible();
    await expect(solarHarvester.getByText(/\+4 \/s/)).toBeVisible();
  });

  test('shows priority order', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByText(/Priority 1/)).toBeVisible();
    await expect(page.getByText(/Priority 2/)).toBeVisible();
  });

  test('shows activate/pause buttons', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    const solarHarvester = page.locator('card').filter({ hasText: 'Solar Harvester' });
    await expect(solarHarvester.getByRole('button', { name: /Pause/ })).toBeVisible();

    const oreReclaimer = page.locator('card').filter({ hasText: 'Ore Reclaimer' });
    await expect(oreReclaimer.getByRole('button', { name: /Activate/ })).toBeVisible();
  });

  test('shows reprioritize buttons', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByRole('button', { name: /↑/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /↓/ })).toBeVisible();
  });

  test('shows deficit warnings when applicable', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/services');

    await expect(page.getByTestId('deficit-warnings')).toBeVisible();
    await expect(page.getByText(/Power deficit in progress/i)).toBeVisible();
  });

  test('shows paused status for services paused by deficit', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/services');

    const surveyUplink = page.locator('card').filter({ hasText: 'Survey Uplink' });
    await expect(surveyUplink.getByText('Paused')).toBeVisible();
  });

  test('shows disabled reasons visible (not hover-only)', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/services');

    const surveyUplink = page.locator('card').filter({ hasText: 'Survey Uplink' });
    await expect(surveyUplink.getByText('Disabled')).toBeVisible();
    await expect(surveyUplink.getByText(/Power deficit/)).toBeVisible();
  });

  test('shows navigation links to other routes', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await expect(page.getByRole('button', { name: /Overview/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Systems/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Planets/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Prestige/i })).toBeVisible();
  });

  test('navigates to systems page', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/services');

    await page.getByRole('button', { name: /Systems/i }).click();

    await expect(page).toHaveURL(/systems/);
  });

  test('renders all-planets fixture with more active services', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'all-planets');
    });
    await page.goto('/services');

    await expect(page.getByText('5 of 5')).toBeVisible();
  });
});
