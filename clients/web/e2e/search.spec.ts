import { expect, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

test.describe('search', () => {
	test('queries a source and renders the results', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.goto('/search');

		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('Manga Seed 07');
		await page.getByRole('button', { name: 'Search', exact: true }).click();

		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 07' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3 })).toHaveCount(1);
		await expect(page.getByText('IMPORT +')).toBeVisible();
		await expect(
			page.getByRole('navigation', { name: 'Search pagination' }).getByText('Page 1'),
		).toBeVisible();
	});

	test('does not swallow a query submitted before the source list arrives', async ({ page }) => {
		await signIn(page, 'searcher');

		let release: (() => void) | undefined;
		await page.route('**/api/sources*', async (route) => {
			await new Promise<void>((resolve) => {
				release = resolve;
			});
			await route.continue();
		});

		await page.goto('/search');
		await expect(page.getByRole('status').getByText('Loading sources…')).toBeVisible();

		await page.getByPlaceholder('Search archive…').fill('Manga Seed 07');
		await page.getByRole('button', { name: 'Search', exact: true }).click();

		await expect(page.getByText('Searching 0 sources')).toBeVisible();

		release?.();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 07' })).toBeVisible();
	});

	test('rejects an empty query instead of doing nothing', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('   ');
		await page.getByRole('button', { name: 'Search', exact: true }).click();

		await expect(page.getByText('Enter something to search for')).toBeVisible();
	});

	test('reports when nothing matches', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('zzz-no-such-title');
		await page.getByRole('button', { name: 'Search', exact: true }).click();

		await expect(page.getByText('No results for “zzz-no-such-title”.')).toBeVisible();
	});

	test('groups results across every source', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByRole('combobox').selectOption('all');
		await page.getByPlaceholder('Search archive…').fill('Seed 01');
		await page.getByRole('button', { name: 'Search', exact: true }).click();

		await expect(page.getByRole('heading', { level: 2, name: /E2E Manga/ })).toBeVisible();
		await expect(page.getByRole('heading', { level: 2, name: /E2E Novel/ })).toBeVisible();
	});

	test('blocks the page behind a progress dialog while importing', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.route('**/api/works', async (route) => {
			await new Promise((resolve) => setTimeout(resolve, 1200));
			await route.continue();
		});
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('Manga Seed 07');
		await page.getByRole('button', { name: 'Search', exact: true }).click();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 07' })).toBeVisible();

		await page.getByRole('button', { name: /Manga Seed 07/ }).click();

		const dialog = page.getByRole('dialog');
		await expect(dialog).toBeVisible();
		await expect(dialog).toHaveAttribute('aria-modal', 'true');
		await expect(dialog).toHaveAttribute('aria-busy', 'true');
		await expect(dialog.getByText('This can take a minute on a long series.')).toBeVisible();
		await expect(dialog.getByText('Importing chapters')).toBeVisible();
		await expect(page.locator('div[inert]')).toHaveCount(1);

		await expect(page).toHaveURL(/\/work\/[0-9a-f-]{36}$/, { timeout: 20000 });
		await expect(page.getByRole('button', { name: 'In Library' })).toBeVisible();
	});

	test('a failed import closes the dialog and reports why', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.route('**/api/works', (route) =>
			route.fulfill({
				status: 502,
				contentType: 'application/json',
				body: JSON.stringify({ code: 'upstream_unavailable', error: 'source is offline' }),
			}),
		);
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('Manga Seed 07');
		await page.getByRole('button', { name: 'Search', exact: true }).click();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 07' })).toBeVisible();

		await page.getByRole('button', { name: /Manga Seed 07/ }).click();

		await expect(page.getByRole('dialog')).toHaveCount(0, { timeout: 5000 });
		await expect(page.getByText('source is offline')).toBeVisible();
		await expect(page.locator('div[inert]')).toHaveCount(0);
	});

	test('imports a result and opens its work page', async ({ page }) => {
		await signIn(page, 'searcher');
		await page.goto('/search');
		await expect(page.getByRole('combobox')).toHaveValue('mv_e2e_manga');
		await page.getByPlaceholder('Search archive…').fill('Manga Seed 07');
		await page.getByRole('button', { name: 'Search', exact: true }).click();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 07' })).toBeVisible();

		await page.getByRole('button', { name: /Manga Seed 07/ }).click();
		await expect(page).toHaveURL(/\/work\/[0-9a-f-]{36}$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Manga Seed 07' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'In Library' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 2, name: /Chapters/ })).toBeVisible();
	});
});
