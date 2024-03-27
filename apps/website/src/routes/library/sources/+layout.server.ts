import { api } from '$lib/axios.server';
import type { MangaSource } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');

	const scrappers: MangaSource[] = await api
		.get(`/api/scrappers`, { headers: { Authorization: token } })
		.then((res) => res.data);

	return {
		scrappers
	};
};
