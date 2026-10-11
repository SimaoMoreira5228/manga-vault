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
