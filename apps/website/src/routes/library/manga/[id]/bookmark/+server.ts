import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

type body = {
	user_id: number;
	manga_id: number;
	category_id: number;
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');
	const body: body = await request.json();

	const resp = await api.post('api/mangas/favorite/add', body, {
		headers: { Authorization: token }
	});

	if (resp.status === 200) {
		return json({ message: 'success' });
	}

	return error(resp.status, resp.statusText);
};
