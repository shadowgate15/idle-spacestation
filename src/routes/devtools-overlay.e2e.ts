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

    await expect(page.getByTestId('devtools-session-panel').getByText(/Current tick: \d+/)).toBeVisible();
    await expect(page.getByTestId('devtools-session-error')).toHaveText('');
  });

  test('reset to starter confirm flow', async ({ page }) => {
    await gotoSeededOverview(page);

    await page.getByTestId('devtools-reset-to-starter-btn').click();
    await expect(page.getByTestId('devtools-reset-confirm-btn')).toBeVisible();

    await page.getByTestId('devtools-reset-confirm-btn').click();

    await expect(page.getByTestId('devtools-overlay')).toBeVisible();
    await expect(page.getByText(/Tick: \d+ · Tier: 1/)).toBeVisible();
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

  test('typing materials across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-materials-input');
    await typeAcrossPollingTicks(input, '12345678901');

    await expect(input).toHaveValue('12345678901');
    await expect(input).toBeFocused();
  });

  test('typing crew total across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-crew-total-input');
    await typeAcrossPollingTicks(input, '12345678901');

    await expect(input).toHaveValue('12345678901');
    await expect(input).toBeFocused();
  });

  test('typing system level across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-system-reactor-core-level');
    await typeAcrossPollingTicks(input, '11111111111');

    await expect(input).toBeFocused();
    await expect(input).toHaveValue('11111111111');
  });

  test('typing assigned crew across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-service-solar-harvester-crew');
    await typeAcrossPollingTicks(input, '12345678901');

    await expect(input).toHaveValue('12345678901');
    await expect(input).toBeFocused();
  });

  test('typing doctrine fragments across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-doctrine-fragments-input');
    await typeAcrossPollingTicks(input, '12345678901');

    await expect(input).toHaveValue('12345678901');
    await expect(input).toBeFocused();
  });

  test('typing advance ticks across polling ticks preserves keystrokes and focus', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-advance-ticks-input');
    await typeAcrossPollingTicks(input, '12345678901');

    await expect(input).toHaveValue('12345678901');
    await expect(input).toBeFocused();
  });

  test('polling resumes after blur', async ({ page }) => {
    await gotoSeededOverview(page);

    const input = page.getByTestId('devtools-materials-input');
    const overlay = page.getByTestId('devtools-overlay');

    await input.click();
    await page.waitForTimeout(2500);
    const tickWhileFocused = await readDisplayedTick(overlay);

    await input.evaluate((el) => (el as HTMLInputElement).blur());

    await expect
      .poll(async () => readDisplayedTick(overlay), { timeout: 5000, intervals: [250, 500, 750] })
      .toBeGreaterThan(tickWhileFocused);
  });

  test('polling does not pause when focus is on Apply button', async ({ page }) => {
    await gotoSeededOverview(page);

    const applyBtn = page.getByTestId('devtools-resources-apply');
    const overlay = page.getByTestId('devtools-overlay');

    await applyBtn.focus();
    await expect(applyBtn).toBeFocused();

    const tickBefore = await readDisplayedTick(overlay);

    await expect
      .poll(async () => readDisplayedTick(overlay), { timeout: 5000, intervals: [250, 500, 750] })
      .toBeGreaterThan(tickBefore);

    await expect(applyBtn).toBeFocused();
  });
});

async function typeAcrossPollingTicks(locator: Locator, text: string) {
  // Focus, select existing value, then type per-key to span > 1 polling interval (1000ms).
  // 11 chars × 100ms = 1100ms — exercises the bug under real polling.
  await locator.click();
  await locator.press('ControlOrMeta+a');
  await locator.pressSequentially(text, { delay: 100 });
}

async function readDisplayedTick(overlay: Locator): Promise<number> {
  const text = (await overlay.textContent()) ?? '';
  const match = text.match(/Tick:\s*(\d+)/);
  if (!match) {
    throw new Error(`Tick counter not found in overlay text: ${text.slice(0, 200)}`);
  }
  return Number(match[1]);
}

async function inputNumericValue(locator: Locator) {
  const value = await locator.inputValue();
  return value !== '' && !Number.isNaN(Number(value));
}
