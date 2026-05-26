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

    const reactorCore = page.locator('[data-slot="card"]').filter({ hasText: 'Reactor Core' });
    await expect(reactorCore).toBeVisible();
    await expect(reactorCore.getByText('Level', { exact: true })).toBeVisible();
    await expect(reactorCore.getByText('1 / 4', { exact: true })).toBeVisible();
    await expect(reactorCore.getByText('Power output', { exact: true })).toBeVisible();
    await expect(reactorCore.getByText('8 power').first()).toBeVisible();
    await expect(reactorCore.getByText('Service power cap', { exact: true })).toBeVisible();
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
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(page.getByTestId('systems-header')).toBeVisible();

    await page.waitForFunction(() => '__gameGateway' in window);
    await page.evaluate(async () => {
      const w = window as unknown as {
        __gameGateway: {
          applyResources: (input: { materials: number; data: number }) => Promise<unknown>;
        };
      };
      await w.__gameGateway.applyResources({ materials: 0, data: 0 });
    });

    await expect(page.getByText(/Needs \d+ Materials/).first()).toBeVisible();
  });

  test('shows upgrade blocked reason for system', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/systems');

    await expect(
      page.getByRole('button', { name: /Upgrade \(\d+ Materials\)/ }).first(),
    ).toBeVisible();
  });

  test('shows disabled upgrade button at max level', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'all-planets');
    });
    await page.goto('/systems');

    await expect(page.getByTestId('systems-header')).toBeVisible();

    await page.waitForFunction(() => '__gameGateway' in window);
    await page.evaluate(async () => {
      const w = window as unknown as {
        __gameGateway: {
          applySystems: (input: {
            systems: Array<{ id: string; level: number }>;
          }) => Promise<unknown>;
        };
      };
      await w.__gameGateway.applySystems({
        systems: [
          { id: 'reactor-core', level: 4 },
          { id: 'habitat-ring', level: 2 },
          { id: 'logistics-spine', level: 3 },
          { id: 'survey-array', level: 2 },
        ],
      });
    });

    const reactorCore = page.locator('[data-slot="card"]').filter({ hasText: 'Reactor Core' });
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

    await expect(page.getByText('3 / 4').first()).toBeVisible();
  });
});
