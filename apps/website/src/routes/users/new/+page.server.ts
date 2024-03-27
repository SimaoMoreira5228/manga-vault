import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getUser } from '$lib/utils.server';

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await getUser(cookies);

	if (user !== null) {
		return redirect(308, '/library');
	}
};
