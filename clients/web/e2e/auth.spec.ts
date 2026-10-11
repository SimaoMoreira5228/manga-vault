import { expect, test } from '@playwright/test';
import { PASSWORD } from './fixtures/sign_in';

test.describe('authentication', () => {
	test('registers, signs out and signs back in', async ({ page }) => {
		const username = `e2e-${Date.now()}`;
		await page.goto('/login');
		await page.getByRole('button', { name: 'No account yet? Create one' }).click();
		await page.getByLabel('Username').fill(username);
		await page.getByLabel('Password').fill(PASSWORD);
		await page.getByRole('button', { name: 'Create account' }).click();

		await expect(page).toHaveURL('/');
		await expect(
			page.getByRole('heading', { level: 1, name: `Welcome back, ${username}` }),
		).toBeVisible();
		await expect(page.getByRole('button', { name: username })).toBeVisible();

		await page.getByRole('button', { name: 'Sign out' }).click();
		await expect(page).toHaveURL('/login');
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

		await page.getByLabel('Username').fill(username);
		await page.getByLabel('Password').fill(PASSWORD);
		await page.getByRole('button', { name: 'Sign in' }).click();

		await expect(page).toHaveURL('/');
		await expect(
			page.getByRole('heading', { level: 1, name: `Welcome back, ${username}` }),
		).toBeVisible();
	});

	test('sends an anonymous visitor on a protected route to the sign-in page', async ({ page }) => {
		await page.goto('/library');
		await expect(page).toHaveURL('/login');
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
	});

	test('rejects a wrong password without signing in', async ({ page }) => {
		await page.goto('/login');
		await page.getByLabel('Username').fill('library_reader');
		await page.getByLabel('Password').fill('not-the-password');
		await page.getByRole('button', { name: 'Sign in' }).click();

		await expect(page.getByRole('alert')).toHaveText('Wrong username or password.');
		await expect(page).toHaveURL('/login');
	});
});
