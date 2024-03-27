import { error } from '@sveltejs/kit';
import { api } from '$lib/axios.server';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const { username, password } = await request.json();

	if (!username || !password) {
		return error(400, 'Username and password are required');
	}

	const resp = await api.post('auth/users/create', {
		username,
		password
	});

	if (resp.status === 200) {
		return new Response(JSON.stringify(resp.data), {
			headers: {
				'content-type': 'application/json'
			}
		});
	} else {
		return error(resp.status, resp.data);
	}
};
