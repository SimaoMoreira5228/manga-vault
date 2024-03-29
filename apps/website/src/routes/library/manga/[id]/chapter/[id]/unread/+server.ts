import { api } from '$lib/axios.server';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

type MarkAsUnRead = {
	user_id: number;
	chapter_id: number;
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const body: MarkAsUnRead = await request.json();

	try {
		const resp = await api.post(
			`/api/user/${body.user_id}/read/${body.chapter_id}/mark-as-unread`,
			body,
			{
				headers: { Authorization: token }
			}
		);

		return json(resp.data);
	} catch (_) {
		return error(500, "Couldn't mark chapter as unread, Probably already marked");
	}
};
