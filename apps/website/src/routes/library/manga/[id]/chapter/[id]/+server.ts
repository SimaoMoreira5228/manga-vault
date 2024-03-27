import { api } from '$lib/axios.server';
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

type body = {
	manga_id: string;
	chapter_id: string;
	chapter_info: {
		title: string;
		pages: number;
	};
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const { manga_id, chapter_id, chapter_info }: body = await request.json();

	const pages: string[] = [];
	for (let i = 1; i <= chapter_info.pages; i++) {
		try {
			const resp = await api.get(`/api/mangas/${manga_id}/chapters/${chapter_id}/pages/${i}`, {
				headers: { Authorization: token }
			});

			pages.push(resp.data);
		} catch (_) {
			pages.push('');
		}
	}

	return json(pages);
};
