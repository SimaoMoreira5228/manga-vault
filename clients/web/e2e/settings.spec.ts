import { expect, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

test.describe('settings', () => {
	test('renders every section for a regular user', async ({ page }) => {
		await signIn(page, 'settings_viewer');
		await page.goto('/settings');

		for (const name of [
			'Appearance',
			'Account',
			'Translation',
			'New chapter notifications',
			'Trackers',
			'Source migration',
			'Account security',
			'Logged-in devices',
			'Data',
		]) {
			await expect(page.getByRole('heading', { level: 2, name, exact: true })).toBeVisible();
		}
		await expect(page.getByRole('heading', { name: 'Registration', exact: true })).toHaveCount(0);
		await expect(page.getByRole('heading', { name: 'Reset a password (operator)' })).toHaveCount(0);
	});

	test('shows registration controls to an admin', async ({ page }) => {
		await signIn(page, 'admin');
		await page.goto('/settings');

		await expect(
			page.getByRole('heading', { level: 2, name: 'Registration', exact: true }),
		).toBeVisible();
		await expect(page.getByLabel('Registration mode')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Generate invite' })).toBeVisible();
		await expect(page.getByText('No invite codes generated.')).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Reset a password (operator)' })).toBeVisible();
	});

	test('switches the appearance theme and remembers it', async ({ page }) => {
		await signIn(page, 'settings_viewer');
		await page.goto('/settings');

		await expect(page.locator('html')).toHaveAttribute('data-theme', 'monochrome-dark');
		await expect(page.getByRole('button', { name: 'Monochrome Dark' })).toHaveAttribute(
			'aria-pressed',
			'true',
		);

		await page.getByRole('button', { name: 'Forest Archive' }).click();
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'forest');
		await expect(page.getByRole('button', { name: 'Forest Archive' })).toHaveAttribute(
			'aria-pressed',
			'true',
		);

		await page.reload();
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'forest');
	});
});
