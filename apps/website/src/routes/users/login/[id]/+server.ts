import { error } from '@sveltejs/kit';
import { api } from '$lib/axios.server';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const { username, password } = await request.json();

	if (!username || !password) {
		return error(400, 'Username and password are required');
	}

	const resp = await api.post('auth/login', {
		username,
		password
	});

	if (resp.status === 200) {
		return new Response(JSON.stringify(resp.data), {
			status: resp.status,
			headers: {
				'content-type': 'application/json',
				'set-cookie':
					resp.headers['set-cookie']?.find((cookie: string) => cookie.startsWith('token=')) || ''
			}
		});
	} else {
		return error(resp.status, resp.data);
	}
};
