import { api } from '$lib/axios.server';
import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getUser, type User } from '$lib/utils.server';

export const load: PageServerLoad = async ({ params, cookies }) => {
	const me = await getUser(cookies);

	if (me !== null) {
		return redirect(308, '/library');
	}

	const res = await api.get(`auth/users/${params.id}`);

	if (!res.data) {
		throw error(404, 'User not found');
	}

	const user: User = res.data;

	return { user };
};
