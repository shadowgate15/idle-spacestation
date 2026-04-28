import { expect, test, type Locator, type Page } from '@playwright/test';

async function seedDevtoolsStarterFixture(page: Page) {
  await page.addInitScript(() => {
    localStorage.setItem('idle-spacestation.transport-mode', 'fixture');
    localStorage.setItem('idle-spacestation.e2e-fixture', 'starter');
    localStorage.setItem('idle-spacestation.devtools-open', 'true');
  });
}

async function gotoSeededOverview(page: Page) {
  await seedDevtoolsStarterFixture(page);
  await page.goto('/');
}

test.describe('Devtools overlay in preview fixture mode', () => {
  test('overlay renders with all sections', async ({ page }) => {
    await gotoSeededOverview(page);

    await expect(page.getByTestId('devtools-overlay')).toBeVisible();
    await expect(page.getByTestId('devtools-resources-section')).toBeVisible();
    await expect(page.getByTestId('devtools-systems-section')).toBeVisible();
    await expect(page.getByTestId('devtools-services-section')).toBeVisible();
    await expect(page.getByTestId('devtools-progression-section')).toBeVisible();
    await expect(page.getByTestId('devtools-session-section')).toBeVisible();
  });

  test('resources panel shows starter values', async ({ page }) => {
    await gotoSeededOverview(page);

    await expect(page.getByTestId('devtools-resources-panel')).toBeVisible();
    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('120');
    await expect(page.getByTestId('devtools-data-input')).toHaveValue('0');
  });

  test('resources apply success', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-materials-input').fill('321');
    await page.getByTestId('devtools-resources-apply').click();

    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('321');
    await expect(page.getByTestId('devtools-resources-error')).toHaveText('');
  });

  test('resources apply with invalid range shows error', async ({ page }) => {
    await gotoSeededOverview(page);

    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('120');
    await page.getByTestId('devtools-materials-input').fill('999999');
    await page.getByTestId('devtools-resources-apply').click();

    await expect(page.getByTestId('devtools-resources-error')).toContainText('invalid_range');
    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('120');
  });

  test('close button hides overlay', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-close-btn').click();

    await expect(page.getByTestId('devtools-overlay')).toHaveCount(0);
  });

  test('session advance ticks', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-advance-ticks-input').fill('5');
    await page.getByTestId('devtools-advance-ticks-btn').click();

    await expect(page.getByTestId('devtools-session-panel').getByText('Current tick: 5')).toBeVisible();
    await expect(page.getByTestId('devtools-session-error')).toHaveText('');
  });

  test('reset to starter confirm flow', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-reset-to-starter-btn').click();
    await expect(page.getByTestId('devtools-reset-confirm-btn')).toBeVisible();

    await page.getByTestId('devtools-reset-confirm-btn').click();

    await expect(page.getByTestId('devtools-overlay')).toBeVisible();
    await expect(page.getByText('Tick: 0 · Tier: 1')).toBeVisible();
  });

  test('resources panel inputs have numeric starter values', async ({ page }) => {
    await gotoSeededOverview(page);

    await expect(inputNumericValue(page.getByTestId('devtools-materials-input'))).resolves.toBe(true);
    await expect(inputNumericValue(page.getByTestId('devtools-data-input'))).resolves.toBe(true);
  });

  test('closing and reopening overlay discards staged drafts', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-materials-input').fill('500');
    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('500');

    await page.getByTestId('devtools-close-btn').click();
    await expect(page.getByTestId('devtools-overlay')).toHaveCount(0);

    await page.evaluate(() => {
      localStorage.setItem('idle-spacestation.devtools-open', 'true');
    });
    await page.reload();

    await expect(page.getByTestId('devtools-overlay')).toBeVisible();
    await expect(page.getByTestId('devtools-materials-input')).toHaveValue('120');
  });
});

async function inputNumericValue(locator: Locator) {
  const value = await locator.inputValue();
  return value !== '' && !Number.isNaN(Number(value));
}
