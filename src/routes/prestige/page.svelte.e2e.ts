import { expect, test } from '@playwright/test';

test.describe('Prestige page with live data', () => {
  test('renders starter fixture as ineligible', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('prestige-hero')).toBeVisible();
    await expect(page.getByText(/Prestige Operations/)).toBeVisible();

    await expect(page.getByTestId('eligibility-panel')).toBeVisible();
    await expect(page.getByText(/Eligibility Status/)).toBeVisible();
    await expect(page.getByText(/Prestige requirements not met/)).toBeVisible();
  });

  test('shows specific reason codes when ineligible', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByText('Station must reach Tier 4')).toBeVisible();
    await expect(page.getByText('Must discover 2 additional planets')).toBeVisible();
    await expect(page.getByText('Need 300s stable power')).toBeVisible();
  });

  test('displays doctrine fragments count', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('doctrine-fragments')).toBeVisible();
    await expect(page.getByText('0')).toBeVisible();
  });

  test('shows reset consequences when revealed', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('reset-consequences')).toBeVisible();
    await expect(page.getByText(/Show what resets vs persists/)).toBeVisible();

    await page.getByText(/Show what resets vs persists/).click();

    await expect(page.getByText(/Discovered planets/)).toBeVisible();
    await expect(page.getByText(/Outcome/)).toBeVisible();
  });

  test('does not show prestige button when ineligible', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('prestige-action')).not.toBeVisible();
  });

  test('renders prestige-ready as eligible', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/prestige');

    await expect(page.getByText(/Prestige is available/)).toBeVisible();
  });

  test('shows Begin Prestige button when eligible', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('prestige-action')).toBeVisible();
    await expect(page.getByText('Begin Prestige')).toBeVisible();
  });

  test('requires confirmation before prestige action', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/prestige');

    await page.getByText('Begin Prestige').click();

    await expect(page.getByText(/This action cannot be undone/)).toBeVisible();
    await expect(page.getByText('Confirm Prestige')).toBeVisible();
    await expect(page.getByText('Cancel')).toBeVisible();
  });

  test('displays unlocked doctrines', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('unlocked-doctrines')).toBeVisible();
    await expect(page.getByText('Efficient Shifts')).toBeVisible();
    await expect(page.getByText('Deep Survey Protocols')).toBeVisible();
  });

  test('shows locked message when no fragments available', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/prestige');

    await expect(page.getByTestId('doctrine-purchase-locked')).toBeVisible();
    await expect(page.getByText(/No fragments available/)).toBeVisible();
  });

  test('displays stable power time progress', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'prestige-ready');
    });
    await page.goto('/prestige');

    await expect(page.getByText(/Stable Power Time/)).toBeVisible();
    await expect(page.getByText(/5:00/)).toBeVisible();
  });
});
