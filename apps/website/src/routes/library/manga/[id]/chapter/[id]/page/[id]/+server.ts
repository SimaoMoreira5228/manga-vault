import { api } from '$lib/axios.server';
import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ request, cookies }) => {
	const token = cookies.get('token');

	const manga_id = request.url.split('/')[5];
	const chapter_id = request.url.split('/')[7];
	const page = request.url.split('/')[9];

	try {
		const resp = await api
			.get(`api/mangas/${manga_id}/chapters/${chapter_id}/pages/${page}`, {
				headers: {
					Authorization: token
				},
				responseType: 'arraybuffer'
			})
			.then((res) => res.data);

		return new Response(resp, {
			headers: {
				'Content-Type': 'image/*',
				'Cache-Control': `public, max-age=${10 * 60}`
			}
		});
	} catch (e) {
		return error(500, 'Internal Server Error');
	}
};
