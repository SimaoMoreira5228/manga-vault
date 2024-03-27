import { api } from '$lib/axios.server';
import type { PageServerLoad } from './$types';
import { getUser } from '$lib/utils.server';
import type { Category } from '$lib/types';

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');
	const user = await getUser(cookies);

	let categories: Category[] = [];
	if (user) {
		categories = await api
			.get(`api/users/${user?.id}/categories`, {
				headers: {
					Authorization: token
				}
			})
			.then((res) => res.data);
	}

	return { categories };
};
