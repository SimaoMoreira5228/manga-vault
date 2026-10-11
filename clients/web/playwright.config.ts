import { defineConfig } from '@playwright/test';

export default defineConfig({
	testDir: './e2e',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: 0,
	workers: 4,
	reporter: [['list']],
	expect: { timeout: 10_000 },
	use: {
		baseURL: 'http://localhost:5173',
		trace: 'retain-on-failure',
	},
	projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
	webServer: [
		{
			command: 'bash e2e/run-api.sh',
			url: 'http://127.0.0.1:8081/api/registration',
			reuseExistingServer: false,
			timeout: 300_000,
		},
		{
			command: 'pnpm run dev -- --port 5173 --strictPort',
			url: 'http://localhost:5173',
			reuseExistingServer: false,
			timeout: 120_000,
			env: { MV_API_URL: 'http://127.0.0.1:8081' },
		},
	],
});
