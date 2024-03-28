import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api } from '$lib/axios.server';

export const PATCH: RequestHandler = async ({ cookies, request }) => {
	const token = cookies.get('token');

	const { user_id, image_id } = await request.json();

	const new_user_id = parseInt(user_id);
	const new_image_id = parseInt(image_id);

	const resp = await api.patch(
		'api/users/image',
		{ user_id: new_user_id, image_id: new_image_id },
		{
			headers: {
				Authorization: token
			}
		}
	);

	return json(resp.data);
};
