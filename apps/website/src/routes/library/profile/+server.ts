import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');
	const formData = await request.formData();

	try {
		const resp = await api.post('api/upload', formData, {
			headers: {
				Authorization: token,
				'Content-Type': 'multipart/form-data'
			}
		});

		return json(resp.data);
	} catch (_) {
		return error(500, 'Internal Server Error');
	}
};
