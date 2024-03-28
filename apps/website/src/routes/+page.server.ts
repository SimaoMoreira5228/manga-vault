import { redirect } from '@sveltejs/kit';
import { api } from '$lib/axios.server';
import type { PageServerLoad } from './$types';
import { getUser } from '$lib/utils.server';
import type { User } from '$lib/types';

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await getUser(cookies);

	if (user !== null) {
		return redirect(308, '/library');
	}

	const users: User[] = await api.get('auth/users').then((res) => res.data);
	return { users };
};
