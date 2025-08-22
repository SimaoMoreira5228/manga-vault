import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		alias: {
			$gql: 'src/gql'
		},
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: 'spa.html',
			precompress: true,
			strict: true
		})
	},
	compilerOptions: {
		experimental: {
			async: true
		}
	}
};

export default config;
