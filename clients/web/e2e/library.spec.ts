import { expect, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

test.describe('library', () => {
	test.beforeEach(async ({ page }) => {
		await signIn(page, 'library_reader');
		await page.goto('/library');
		await expect(page.getByRole('heading', { level: 1, name: 'Library' })).toBeVisible();
	});

	test('shows the seeded works as cover cards', async ({ page }) => {
		await expect(page.getByText('60 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 10' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 15' })).toHaveCount(0);
		await expect(page.getByLabel('Category for Manga Seed 01')).toBeVisible();
	});

	test('loads the rest of the library as you scroll', async ({ page }) => {
		const titles = page.getByRole('heading', { level: 3 });
		const initial = await titles.count();
		expect(initial).toBeLessThan(60);

		await expect(async () => {
			await page.mouse.wheel(0, 20000);
			await page.waitForTimeout(300);
			const grown = await titles.count();
			if (grown <= initial) throw new Error('the list never grew');
		}).toPass({ timeout: 25000 });

		await expect(titles).toHaveCount(60);
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 15' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();
		await expect(page.getByText('60 works')).toBeVisible();
	});

	test('filters by kind and by title', async ({ page }) => {
		await page.getByRole('button', { name: 'Mangas' }).click();
		await expect(page.getByText('40 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 01' })).toHaveCount(0);

		await page.getByRole('button', { name: 'Novels' }).click();
		await expect(page.getByText('20 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 01' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Manga Seed 01' })).toHaveCount(0);

		await page.getByRole('button', { name: 'All', exact: true }).first().click();
		await expect(page.getByText('60 works')).toBeVisible();

		await page.getByPlaceholder('Filter titles…').fill('Novel Seed 12');
		await expect(page.getByText('1 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3, name: 'Novel Seed 12' })).toBeVisible();

		await page.getByPlaceholder('Filter titles…').fill('');
		await expect(page.getByText('60 works')).toBeVisible();
	});

	test('searches, filters and paginates with a cursor-shaped response', async ({ page }) => {
		await page.route('**/api/library?*', async (route) => {
			const url = new URL(route.request().url());
			url.searchParams.set('limit', '200');
			url.searchParams.delete('after');
			const response = await route.fetch({ url: url.toString() });
			const { entries, categories, total, limit, next_cursor } = await response.json();
			await route.fulfill({ json: { entries, categories, total, limit, next_cursor } });
		});
		await page.reload();
		await expect(page.getByText('60 works')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Check all for updates' })).toBeVisible();
		await page.mouse.wheel(0, 20000);
		await expect(page.getByRole('heading', { level: 3 })).toHaveCount(60);

		await page.getByRole('button', { name: 'Mangas' }).click();
		await expect(page.getByText('40 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3 })).toHaveCount(40);
		await page.getByPlaceholder('Filter titles…').fill('Manga Seed 12');
		await expect(page.getByText('1 works')).toBeVisible();
		await expect(page.getByRole('heading', { level: 3 })).toHaveText('Manga Seed 12');
		await page.getByPlaceholder('Filter titles…').fill('no matching title');
		await expect(page.getByText('0 works')).toBeVisible();
		await expect(page.getByPlaceholder('Filter titles…')).toBeVisible();
		await page.getByPlaceholder('Filter titles…').fill('');
		await page.getByRole('button', { name: 'All', exact: true }).first().click();
		await expect(page.getByText('60 works')).toBeVisible();
	});

	test('sorts the grid', async ({ page }) => {
		const cards = page.getByRole('heading', { level: 3 });
		await expect(cards.first()).toHaveText('Novel Seed 10');

		await page.getByLabel('Sort library').selectOption('title');
		await expect(cards.first()).toHaveText('Manga Seed 01');

		await page.getByLabel('Sort library').selectOption('unread-desc');
		await expect(cards.first()).toHaveText('Manga Seed 01');
	});

	test('shows unread chapter counts from reading progress', async ({ page }) => {
		const almostRead = page.locator('div.relative', {
			has: page.getByLabel('Category for Manga Seed 01'),
		});
		await expect(almostRead.getByText('3', { exact: true })).toBeVisible();

		const partlyRead = page.locator('div.relative', {
			has: page.getByLabel('Category for Manga Seed 02'),
		});
		await expect(partlyRead.getByText('1', { exact: true })).toBeVisible();
	});
});
