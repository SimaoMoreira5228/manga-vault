import { api } from '$lib/axios.server';
import type { allSearchedMangaItems } from '$lib/types';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const { title } = await request.json();

	const mangaItems: allSearchedMangaItems[] = await api
		.get(`api/mangas/search/${title}/all`, {
			headers: {
				Authorization: token
			}
		})
		.then((res) => res.data);

	if (mangaItems.length === 0) {
		return error(404, 'Not found');
	}

	return json(mangaItems);
};
