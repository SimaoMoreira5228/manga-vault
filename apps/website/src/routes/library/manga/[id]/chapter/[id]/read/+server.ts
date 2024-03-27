import { api } from '$lib/axios.server';
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

type MarkAsRead = {
	user_id: number;
	chapter_id: number;
	manga_id: number;
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const body: MarkAsRead = await request.json();

	const resp = await api.post(
		`/api/user/${body.user_id}/read/${body.chapter_id}/mark-as-read`,
		body,
		{
			headers: { Authorization: token }
		}
	);

	return json(resp.data);
};
