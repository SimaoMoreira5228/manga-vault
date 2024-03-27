import { api } from '$lib/axios.server';
import { getUser } from '$lib/utils.server';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

type ChapterInfo = {
	title: string;
	pages: number;
	next_chapter: number | null;
	previous_chapter: number | null;
};

export const load: PageServerLoad = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const manga_id = request.url.split('/')[5];
	const chapter_id = request.url.split('/')[7];

	let chapter_info: ChapterInfo;
	try {
		chapter_info = await api
			.get(`/api/mangas/${manga_id}/chapters/${chapter_id}`, { headers: { Authorization: token } })
			.then((res) => res.data);
	} catch (e) {
		return error(404, 'Chapter info Not found');
	}

	const user = await getUser(cookies);

	const chapter = {
		manga_id,
		chapter_id,
		chapter_info
	};

	return {
		user,
		chapter
	};
};
