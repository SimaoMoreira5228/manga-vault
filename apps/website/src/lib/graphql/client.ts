import { env } from "$env/dynamic/public";
import { cacheExchange, createClient, fetchExchange } from "@urql/svelte";

export const client = createClient({
	url: env.PUBLIC_API_URL,
	fetchOptions: () => {
		return { credentials: "include" };
	},
	exchanges: [cacheExchange, fetchExchange],
});
