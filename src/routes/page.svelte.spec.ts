import { afterEach, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { render } from 'vitest-browser-svelte';
import Page from './+page.svelte';

afterEach(() => {
	clearMocks();
});

describe('+page.svelte greet behavior', () => {
	it('renders heading, input, and greet button', async () => {
		render(Page);
		await expect.element(page.getByRole('heading', { level: 1 })).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: /greet/i })).toBeInTheDocument();
	});

	it('invokes greet command and shows result via mockIPC', async () => {
		mockIPC((cmd) => {
			if (cmd === 'greet') return "Hello, Taylor! You've been greeted from Rust!";
		});
		render(Page);
		const input = page.getByPlaceholder('Enter a name...');
		await input.fill('Taylor');
		await page.getByRole('button', { name: /greet/i }).click();
		await expect
			.element(page.getByText("Hello, Taylor! You've been greeted from Rust!"))
			.toBeInTheDocument();
	});

	it('submits with empty input and shows empty-name greeting via mockIPC', async () => {
		mockIPC((cmd) => {
			if (cmd === 'greet') return "Hello, ! You've been greeted from Rust!";
		});
		render(Page);
		await page.getByRole('button', { name: /greet/i }).click();
		await expect
			.element(page.getByText("Hello, ! You've been greeted from Rust!"))
			.toBeInTheDocument();
	});
});
