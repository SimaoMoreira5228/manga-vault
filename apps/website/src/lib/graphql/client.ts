import { createClient, cacheExchange, fetchExchange } from '@urql/svelte';

export const client = createClient({
	url: import.meta.env.VITE_API_URL,
	fetchOptions: () => {
		return {
			credentials: 'include'
		};
	},
	exchanges: [cacheExchange, fetchExchange]
});
