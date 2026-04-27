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

    await expect(page.getByTestId('resource-strip')).toBeVisible();
    await expect(page.getByText(/Power/)).toBeVisible();
    await expect(page.getByText(/Materials/)).toBeVisible();
    await expect(page.getByText(/Data/)).toBeVisible();

    await expect(page.getByTestId('overview-panel')).toBeVisible();
    await expect(page.getByText(/Tier 1/)).toBeVisible();
    await expect(page.getByText(/Service Utilization/)).toBeVisible();

    await expect(page.getByTestId('survey-panel')).toBeVisible();
  });

  test('displays deficit warnings when power is negative', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/');

    await expect(page.getByTestId('deficit-warnings')).toBeVisible();
    await expect(page.getByText(/Power deficit in progress/i)).toBeVisible();
  });

  test('renders prestige-ready fixture data correctly', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/');

    await expect(page.getByText(/Aurora Pier/)).toBeVisible();
    await expect(page.getByText(/Tier 1/)).toBeVisible();
    await expect(page.getByTestId('survey-panel')).toBeVisible();
  });

  test('displays navigation links to other routes', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/');

    await expect(page.getByRole('button', { name: /Systems/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Services/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Planets/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Prestige/i })).toBeVisible();
  });
});
