import { expect, type Page } from '@playwright/test';

export const PASSWORD = 'e2e-password-1234';

export async function signIn(page: Page, username: string): Promise<void> {
	const response = await page.request.post('/api/auth/login', {
		data: { username, password: PASSWORD },
	});
	expect(response.ok()).toBeTruthy();
}
