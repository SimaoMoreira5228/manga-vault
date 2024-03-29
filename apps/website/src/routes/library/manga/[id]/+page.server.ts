import { api } from '$lib/axios.server';
import type { Category } from '$lib/types';
import { getUser } from '$lib/utils.server';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, params }) => {
	const token = cookies.get('token');
	const user = await getUser(cookies);

	let categories: Category[] = [];
	if (user !== null) {
		categories = await api
			.get(`/api/users/${user.id}/categories`, { headers: { Authorization: token } })
			.then((res) => res.data);
	}

	return {
		user,
		mangaId: params.id,
		categories
	};
};
