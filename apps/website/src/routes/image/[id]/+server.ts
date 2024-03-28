import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

export const GET: RequestHandler = async ({ params }) => {
	try {
		const resp = await api
			.get(`files/image/${params.id}`, {
				responseType: 'arraybuffer'
			})
			.then((res) => res.data);

		return new Response(resp, {
			headers: {
				'Content-Type': 'image/jpeg'
			}
		});
	} catch (_) {
		return error(500, 'Internal Server Error');
	}
};
