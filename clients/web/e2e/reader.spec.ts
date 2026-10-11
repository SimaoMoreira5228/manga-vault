import { expect, type Page, test } from '@playwright/test';
import { signIn } from './fixtures/sign_in';

async function openReader(page: Page, title: string): Promise<void> {
	await signIn(page, 'reader');
	await page.goto('/library');
	await page.getByRole('heading', { level: 3, name: title }).click();
	await expect(page.getByRole('heading', { level: 1, name: title })).toBeVisible();
	const chapterRow = page.locator('li').filter({
		has: page.getByLabel(/^Mark as (read|unread): Chapter 1$/),
	});
	await chapterRow.getByRole('link').click();
	await expect(page).toHaveURL(/\/reader\/[0-9a-f-]{36}\?work=/);
	await expect(page.getByRole('heading', { level: 1, name: `${title}: Chapter 1` })).toBeVisible();
}

test.describe('reader', () => {
	test('opens a manga chapter with image options only', async ({ page }) => {
		await openReader(page, 'Manga Seed 07');
		await expect(
			page.getByRole('heading', { level: 1, name: 'Manga Seed 07: Chapter 1' }),
		).toBeVisible();

		await page.getByRole('button', { name: 'Reader settings' }).click();
		await expect(page.getByRole('heading', { name: 'Reader settings' })).toBeVisible();
		await expect(page.getByLabel('Side margins as a percentage of the width')).toBeVisible();
		await expect(page.getByLabel('Gap between images as a percentage of the width')).toBeVisible();
		await expect(page.getByLabel('Font size in pixels')).toHaveCount(0);
		await expect(page.getByLabel('Line spacing')).toHaveCount(0);
	});

	test('persists manga reader settings across a reload', async ({ page }) => {
		await openReader(page, 'Manga Seed 07');
		await page.getByRole('button', { name: 'Reader settings' }).click();
		const margins = page.getByLabel('Side margins as a percentage of the width');
		await expect(margins).toHaveValue('0');
		await margins.fill('10');
		await expect(page.getByText('10%')).toBeVisible();

		await page.reload();
		await page.getByRole('button', { name: 'Reader settings' }).click();
		await expect(page.getByLabel('Side margins as a percentage of the width')).toHaveValue('10');
	});

	test('closes the settings sheet and then leaves the reader on Escape', async ({ page }) => {
		await openReader(page, 'Manga Seed 07');
		await page.getByRole('button', { name: 'Reader settings' }).click();
		await expect(page.getByRole('heading', { name: 'Reader settings' })).toBeVisible();

		await page.keyboard.press('Escape');
		await expect(page.getByRole('heading', { name: 'Reader settings' })).toHaveCount(0);
		await expect(page).toHaveURL(/\/reader\//);

		await page.keyboard.press('Escape');
		await expect(page).toHaveURL(/\/work\//);
	});

	test('opens a novel chapter with text options only', async ({ page }) => {
		await openReader(page, 'Novel Seed 07');
		await expect(
			page.getByRole('heading', { level: 1, name: 'Novel Seed 07: Chapter 1' }),
		).toBeVisible();
		await expect(page.locator('article')).toBeVisible();

		await page.getByRole('button', { name: 'Reader settings' }).click();
		await expect(page.getByRole('heading', { name: 'Reader settings' })).toBeVisible();
		await expect(page.getByLabel('Font size in pixels')).toBeVisible();
		await expect(page.getByLabel('Line spacing')).toBeVisible();
		await expect(page.getByText('Right to left')).toBeVisible();
		await expect(page.getByLabel('Side margins as a percentage of the width')).toHaveCount(0);
		await expect(page.getByText('Images', { exact: true })).toHaveCount(0);
	});

	test('persists the novel font size and applies it to the text', async ({ page }) => {
		await openReader(page, 'Novel Seed 07');
		await expect(page.locator('article')).toHaveCSS('font-size', '16px');

		await page.getByRole('button', { name: 'Reader settings' }).click();
		const fontSize = page.getByLabel('Font size in pixels');
		await expect(fontSize).toHaveValue('16');
		await fontSize.fill('24');
		await expect(page.getByText('24px')).toBeVisible();
		await expect(page.locator('article')).toHaveCSS('font-size', '24px');

		await page.reload();
		await expect(page.locator('article')).toHaveCSS('font-size', '24px');
		await page.getByRole('button', { name: 'Reader settings' }).click();
		await expect(page.getByLabel('Font size in pixels')).toHaveValue('24');
	});
});
