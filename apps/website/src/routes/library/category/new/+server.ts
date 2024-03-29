import { api } from '$lib/axios.server';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

type NewCategory = {
	user_id: number;
	name: string;
};

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const body: NewCategory = await request.json();

	try {
		const resp = await api.post(`/api/categories/create`, body, {
			headers: { Authorization: token }
		});

		return json(resp.data);
	} catch (_) {
		return error(500, "Couldn't create category");
	}
};
