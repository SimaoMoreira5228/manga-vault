import { getUser } from '$lib/utils.server';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await getUser(cookies);

	if (user === null) {
		throw redirect(304, '/');
	}

	return {
		user
	};
};
