import { expect, test } from '@playwright/test';

test.describe('Overview page with live data', () => {
  test('renders starter fixture data correctly', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/');

    await expect(page.getByTestId('home-hero')).toBeVisible();
    await expect(page.getByText(/Station Command/)).toBeVisible();
    await expect(page.getByText('Solstice Anchor')).toBeVisible();

    const resourceStrip = page.getByTestId('resource-strip');
    await expect(resourceStrip).toBeVisible();
    await expect(resourceStrip.getByText('Materials')).toBeVisible();
    await expect(resourceStrip.getByText('Data')).toBeVisible();

    const crewPowerPanel = page.getByTestId('crew-power-panel');
    await expect(crewPowerPanel).toBeVisible();
    await expect(crewPowerPanel.getByText('Power', { exact: true })).toBeVisible();
    await expect(crewPowerPanel.getByText('Crew', { exact: true })).toBeVisible();

    const overviewPanel = page.getByTestId('overview-panel');
    await expect(overviewPanel).toBeVisible();
    await expect(overviewPanel.getByRole('heading', { name: /Tier 1/ })).toBeVisible();
    await expect(
      page.getByTestId('station-stats').getByRole('heading', { name: /Service Utilization/ }),
    ).toBeVisible();

    await expect(page.getByTestId('survey-panel')).toBeVisible();
  });

  test('displays deficit warnings when power is negative', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/');

    const deficitWarnings = page.getByTestId('deficit-warnings');
    await expect(deficitWarnings).toBeVisible();
    await expect(
      deficitWarnings.getByRole('heading', { name: /Power deficit in progress/i }),
    ).toBeVisible();
  });

  test('renders prestige-ready fixture data correctly', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/');

    await expect(page.getByText(/Aurora Pier/)).toBeVisible();
    await expect(
      page.getByTestId('overview-panel').getByRole('heading', { name: /Tier \d+/ }),
    ).toBeVisible();
    await expect(page.getByTestId('survey-panel')).toBeVisible();
  });

  test('displays navigation links to other routes', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/');

    const header = page.getByTestId('game-header');
    await expect(header.getByRole('link', { name: /Systems/i })).toBeVisible();
    await expect(header.getByRole('link', { name: /Services/i })).toBeVisible();
    await expect(header.getByRole('link', { name: /Planets/i })).toBeVisible();
    await expect(header.getByRole('link', { name: /Prestige/i })).toBeVisible();
  });
});
