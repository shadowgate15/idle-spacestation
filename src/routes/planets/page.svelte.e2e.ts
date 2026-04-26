import { expect, test } from '@playwright/test';

test.describe('Planets page with live data', () => {
  test('renders starter fixture with discovered planets', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/planets');

    await expect(page.getByTestId('planets-hero')).toBeVisible();
    await expect(page.getByText(/Planetary Operations/)).toBeVisible();

    await expect(page.getByTestId('survey-progress')).toBeVisible();
    await expect(page.getByText(/Survey Progress/)).toBeVisible();

    await expect(page.getByTestId('planets-list')).toBeVisible();
    await expect(page.getByTestId('planet-solstice-anchor')).toBeVisible();
    await expect(page.getByText('Solstice Anchor')).toBeVisible();
    await expect(page.getByTestId('planet-cinder-forge')).toBeVisible();
    await expect(page.getByTestId('planet-aurora-pier')).toBeVisible();
  });

  test('shows active planet status', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/planets');

    await expect(page.getByTestId('planet-solstice-anchor')).toBeVisible();
    await expect(page.getByText('Active', { exact: true })).toBeVisible();
  });

  test('shows survey progress for next discovery', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/planets');

    await expect(page.getByText(/Current/)).toBeVisible();
    await expect(page.getByText(/Next Target/)).toBeVisible();
    await expect(page.getByText('Cinder Forge')).toBeVisible();
    await expect(page.getByText('600')).toBeVisible();
  });

  test('displays planet modifiers', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    });
    await page.goto('/planets');

    await expect(page.getByText('+10% Crew efficiency')).toBeVisible();
    await expect(page.getByText('-10% Data output')).toBeVisible();
  });

  test('renders all-planets fixture with multiple discoveries', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('idle-spacestation.e2e-fixture', 'all-planets');
    });
    await page.goto('/planets');

    await expect(page.getByTestId('planet-solstice-anchor')).toBeVisible();
    await expect(page.getByTestId('planet-cinder-forge')).toBeVisible();
    await expect(page.getByTestId('planet-aurora-pier')).toBeVisible();

    await expect(page.getByText('Selectable', { exact: true })).toBeVisible();
  });
});
