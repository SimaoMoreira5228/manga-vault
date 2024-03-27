import { json } from '@sveltejs/kit';
import type { MangaItem } from '$lib/types';
import { api } from '$lib/axios.server';
import type { RequestHandler } from './$types';
import { getUser } from '$lib/utils.server';

export const GET: RequestHandler = async ({ cookies, params }) => {
	const token = cookies.get('token');
	const user = await getUser(cookies);

	const favorites: MangaItem[] = await api
		.get(`/api/mangas/${user?.id}/categories/${params.id}/favorites`, {
			headers: { Authorization: token }
		})
		.then((res) => res.data);

	return json(favorites);
};
