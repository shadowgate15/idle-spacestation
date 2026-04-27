import { expect, test } from '@playwright/test';

test.describe('Systems page with live data', () => {
  test('renders all 4 systems for starter fixture', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(page.getByTestId('systems-header')).toBeVisible();
    await expect(page.getByText(/Station Systems/)).toBeVisible();

    await expect(page.getByText('Reactor Core')).toBeVisible();
    await expect(page.getByText('Habitat Ring')).toBeVisible();
    await expect(page.getByText('Logistics Spine')).toBeVisible();
    await expect(page.getByText('Survey Array')).toBeVisible();
  });

  test('shows level and cap values for each system', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(page.getByText('Reactor Core')).toBeVisible();
    await expect(page.getByText('Level 1 / 4')).toBeVisible();
    await expect(page.getByText('Power output')).toBeVisible();
    await expect(page.getByText('8 power')).toBeVisible();
    await expect(page.getByText('Service power cap')).toBeVisible();
    await expect(page.getByText('8 power')).toBeVisible();
  });

  test('shows upgrade buttons with costs', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(page.getByRole('button', { name: /Upgrade/ })).toHaveCount(4);
    await expect(page.getByRole('button', { name: /40 Materials/ })).toBeVisible();
  });

  test('shows upgrade buttons disabled when insufficient materials', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/systems');

    await expect(page.getByText(/Needs/)).toBeVisible();
  });

  test('shows upgrade blocked reason for system', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'deficit');
    });
    await page.goto('/systems');

    await expect(page.getByText(/Materials/)).toBeVisible();
  });

  test('shows disabled upgrade button at max level', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'all-planets');
    });
    await page.goto('/systems');

    const reactorCore = page.locator('card').filter({ hasText: 'Reactor Core' });
    await expect(reactorCore.getByRole('button', { name: /Max Level/ })).toBeVisible();
  });

  test('shows navigation links to other routes', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(page.getByRole('button', { name: /Overview/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Services/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Planets/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Prestige/i })).toBeVisible();
  });

  test('navigates to services page', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await page.getByRole('button', { name: /Services/i }).click();

    await expect(page).toHaveURL(/services/);
  });

  test('renders all-planets fixture with higher system levels', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'all-planets');
    });
    await page.goto('/systems');

    await expect(page.getByText('Level 3 / 4')).toBeVisible();
  });
});
