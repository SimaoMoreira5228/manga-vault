import { getUser } from '$lib/utils.server';
import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies }) => {
	const user = await getUser(cookies);

	if (user === null) {
		return redirect(308, '/');
	}
};
