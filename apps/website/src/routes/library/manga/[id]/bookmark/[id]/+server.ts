import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

export const DELETE: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const manga_id = request.url.split('/')[5];
	const user_id = request.url.split('/')[7];

	const resp = await api.delete(`api/mangas/favorite/remove/${user_id}/${manga_id}`, {
		headers: { Authorization: token }
	});

	if (resp.status === 200) {
		return json({ message: 'success' });
	}

	return error(resp.status, resp.statusText);
};
