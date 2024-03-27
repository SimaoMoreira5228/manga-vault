import { api } from '$lib/axios.server';
import type { Category, FavoitesMangaItem, MangaPage, ReadChapter } from '$lib/types';
import { getUser } from '$lib/utils.server';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, params }) => {
	const token = cookies.get('token');
	const user = await getUser(cookies);

	let mangaPage: MangaPage;

	try {
		mangaPage = await api
			.get(`/api/mangas/scrape/${params.id}`, { headers: { Authorization: token } })
			.then((res) => res.data);
	} catch (e) {
		return error(404, 'Not found');
	}

	let isBookmarked = false;
	if (user !== null) {
		const data: FavoitesMangaItem[] = await api
			.get(`/api/mangas/${user.id}/favorites`, { headers: { Authorization: token } })
			.then((res) => res.data);

		isBookmarked = data.some((manga) => manga.id === parseInt(params.id));
	}

	let categories: Category[] = [];
	if (user !== null) {
		categories = await api
			.get(`/api/users/${user.id}/categories`, { headers: { Authorization: token } })
			.then((res) => res.data);
	}

	let readChapters: ReadChapter[] = [];
	if (user !== null) {
		readChapters = await api
			.get(`/api/user/${user.id}/manga/${params.id}/read-chapters`, {
				headers: { Authorization: token }
			})
			.then((res) => res.data);
	}

	const newMangaPage = {
		id: params.id,
		...mangaPage,
		chapters: mangaPage.chapters.reverse(),
		isBookmarked,
		readChaptersIds: readChapters.map((readChapter) => readChapter.chapter_id)
	};

	return {
		user,
		mangaPage: newMangaPage,
		categories
	};
};
