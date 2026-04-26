import { expect, test } from '@playwright/test';

test('root page enforces dark mode even when browser prefers light', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto('/');
  const hasDark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
  expect(hasDark).toBe(true);
});

test('root page shows tactical shell anchors and primary action', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('game-shell')).toBeVisible();
  await expect(page.getByTestId('game-header')).toBeVisible();
  await expect(page.locator('nav a[href="#overview-panel"]')).toBeVisible();
  await expect(page.locator('nav a[href="#systems-panel"]')).toBeVisible();
  await expect(page.locator('nav a[href="#alerts-panel"]')).toBeVisible();
  await expect(page.getByTestId('primary-action')).toBeVisible();
  await page.getByTestId('primary-action').focus();
  const focused = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
  expect(focused).toBe('primary-action');
});

test('root page keeps primary controls in the station blue family', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('primary-action')).toBeVisible();
  const ctaBg = await page.evaluate(() => {
    const el = document.querySelector('[data-testid="primary-action"]');
    return el ? window.getComputedStyle(el).backgroundColor : '';
  });
  // #0984e3 = rgb(9, 132, 227) — assert it contains a blue-family color
  expect(ctaBg).toMatch(/rgb\(9,\s*132,\s*227\)|rgb\(\d+,\s*\d+,\s*2[0-9]{2}\)/);
});

test('root page stays usable at 960x700', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 700 });
  await page.goto('/');
  await expect(page.getByTestId('game-shell')).toBeVisible();
  await expect(page.getByTestId('primary-action')).toBeVisible();
  const hasOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > window.innerWidth + 1,
  );
  expect(hasOverflow).toBe(false);
});
