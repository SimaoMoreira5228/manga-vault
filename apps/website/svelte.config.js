import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		alias: {
			"$gql": "src/gql",
		},
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			// SPA index file
			fallback: 'spa.html',
			strict: true,
			precompress: true
		})
	},
	compilerOptions: {
		experimental: {
			async: true
		}
	}
};

export default config;
