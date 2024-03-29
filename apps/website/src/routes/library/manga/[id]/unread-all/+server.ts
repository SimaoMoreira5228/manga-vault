import { api } from '$lib/axios.server';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

type MarkAllAsUnRead = {
	user_id: number;
	manga_id: number;
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const body: MarkAllAsUnRead = await request.json();

	try {
		const resp = await api.post(
			`/api/user/${body.user_id}/read/${body.manga_id}/mark-all-as-unread`,
			body,
			{
				headers: { Authorization: token }
			}
		);

		return json(resp.data);
	} catch (_) {
		return error(500, "Couldn't mark all chapters as unread, Probably already marked");
	}
};
