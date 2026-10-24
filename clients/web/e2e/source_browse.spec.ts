import { expect, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

test.describe('source browse', () => {
	test('renders trending by default', async ({ page }) => {
		await signIn(page, 'source_browser');
		await page.goto('/sources/mv_e2e_manga');

		await expect(page.getByRole('heading', { level: 1, name: 'E2E Manga' })).toBeVisible();
		await expect(page.getByRole('tab', { name: 'trending' })).toHaveAttribute(
			'aria-selected',
			'true',
		);
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();
	});

	test('switches to latest and loads more pages', async ({ page }) => {
		await signIn(page, 'source_browser');
		await page.goto('/sources/mv_e2e_manga');

		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();

		await page.getByRole('tab', { name: 'latest' }).click();
		await expect(page.getByRole('tab', { name: 'latest' })).toHaveAttribute(
			'aria-selected',
			'true',
		);
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();

		const loadMore = page.getByRole('button', { name: 'Load more' });
		if (await loadMore.isVisible()) await loadMore.click();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 11' })).toBeVisible();
	});

	test('reports when the source is not installed', async ({ page }) => {
		await signIn(page, 'source_browser');
		await page.goto('/sources/no-such-source');

		await expect(page.getByText('is not installed')).toBeVisible();
	});

	test('explore headings deep-link to the per-source page', async ({ page }) => {
		await signIn(page, 'source_browser');
		await page.goto('/');

		const sourceLink = page.getByRole('link', { name: /E2E Manga/ }).first();
		await expect(sourceLink).toHaveAttribute('href', '/sources/mv_e2e_manga');
		await sourceLink.click();
		await expect(page).toHaveURL(/\/sources\/mv_e2e_manga/);
		await expect(page.getByRole('heading', { level: 1, name: 'E2E Manga' })).toBeVisible();
	});
});
