import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params }) => {
	const url = Buffer.from(params.url, 'base64').toString('utf-8');

	try {
		const resp = await fetch(url).then((res) => res.arrayBuffer());

		return new Response(resp, {
			headers: {
				'Content-Type': 'image/*',
				'Cache-Control': `public, max-age=${10 * 60}`
			}
		});
	} catch (_) {
		return error(500, 'Internal Server Error');
	}
};
