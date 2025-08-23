import { createClient, cacheExchange, fetchExchange } from '@urql/svelte';
import { env } from '$env/dynamic/public';

export const client = createClient({
	url: env.PUBLIC_API_URL,
	fetchOptions: () => {
		return {
			credentials: 'include'
		};
	},
	exchanges: [cacheExchange, fetchExchange]
});
