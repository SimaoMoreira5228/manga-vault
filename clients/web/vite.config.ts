import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import Icons from 'unplugin-icons/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		Icons({ compiler: 'svelte' }),
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
				experimental: {
					async: true,
				}
			},
			adapter: adapter({ fallback: 'index.html', strict: false }),
		}),
	],
	server: {
		proxy: {
			'/api': {
				target: process.env.MV_API_URL ?? 'http://127.0.0.1:18080',
				changeOrigin: true,
			},
		},
	},
});
