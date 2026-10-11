import { expect, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

test.describe('keyboard shortcuts', () => {
	test.beforeEach(async ({ page }) => {
		await signIn(page, 'shortcuts_user');
		await page.goto('/settings');
		await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();
	});

	test('slash jumps to search and focuses the field', async ({ page }) => {
		await page.keyboard.press('/');
		await expect(page).toHaveURL('/search');
		await expect(page.locator('#search-query')).toBeFocused();
	});

	test('question mark toggles the shortcut help', async ({ page }) => {
		await page.keyboard.press('?');
		const help = page.getByRole('dialog');
		await expect(help).toBeVisible();
		await expect(help.getByRole('heading', { name: 'Keyboard shortcuts' })).toBeVisible();
		await expect(help.getByText('Focus search')).toBeVisible();

		await page.keyboard.press('Escape');
		await expect(help).toHaveCount(0);

		await page.keyboard.press('?');
		await expect(help).toBeVisible();
		await help.getByRole('button', { name: 'Close' }).click();
		await expect(help).toHaveCount(0);
	});

	test('g then l opens the library', async ({ page }) => {
		await page.keyboard.press('g');
		await page.keyboard.press('l');
		await expect(page).toHaveURL('/library');
		await expect(page.getByRole('heading', { level: 1, name: 'Library' })).toBeVisible();
	});
});

test.describe('keyboard shortcuts while typing', () => {
	test('stay out of the way in a text field', async ({ page }) => {
		await page.goto('/login');
		await expect(page.getByLabel('Username')).toBeVisible();
		await page.getByLabel('Username').click();
		await page.keyboard.type('gl/');

		await expect(page).toHaveURL('/login');
		await expect(page.getByLabel('Username')).toHaveValue('gl/');
	});
});
