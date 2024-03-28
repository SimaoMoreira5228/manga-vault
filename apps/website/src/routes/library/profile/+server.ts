import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

export const POST: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');
	const formData = await request.formData();

	const resp = await api.post('api/upload', formData, {
		headers: {
			Authorization: token,
			'Content-Type': 'multipart/form-data'
		}
	});

	if (resp.status !== 200) {
		return error(resp.status, resp.statusText);
	}

	const data = await resp.data;

	return json(data);
};
