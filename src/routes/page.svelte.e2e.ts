import { expect, test } from '@playwright/test';

test('root page renders expected heading', async ({ page }) => {
	await page.goto('/');
	await expect(page.locator('h1')).toBeVisible();
});

test('root page has name input', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByPlaceholder('Enter a name...')).toBeVisible();
});

test('root page has Greet button', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByRole('button', { name: /greet/i })).toBeVisible();
});
