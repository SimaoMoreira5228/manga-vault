import { api } from '$lib/axios.server';
import type { allSearchedMangaItems } from '$lib/types';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const url = request.url.split('/');
	const title = url[5];
	const scrapper = url[7];
	const page = url[9];

	const mangaItems: allSearchedMangaItems[] = await api
		.get(`api/mangas/search/${title}/${scrapper}/${page}`, {
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
