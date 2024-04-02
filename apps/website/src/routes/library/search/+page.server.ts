import { api } from '$lib/axios.server';
import type { PageServerLoad } from './$types';
import { getUser } from '$lib/utils.server';

type Scrappers = {
	id: string;
	name: string;
	img_url: string;
};

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');
	const user = await getUser(cookies);

	let scrappers: Scrappers[] = [];
	if (user) {
		scrappers = await api
			.get(`api/scrappers`, {
				headers: {
					Authorization: token
				}
			})
			.then((res) => res.data);
	}

	return { scrappers };
};
