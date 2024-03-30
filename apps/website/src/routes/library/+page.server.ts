import { api } from '$lib/axios.server';
import type { PageServerLoad } from './$types';
import { getUser } from '$lib/utils.server';
import type { Category } from '$lib/types';

type WebsocketInfo = {
	websocket_ip: string;
	websocket_port: number;
};

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

	const WebsocketInfo: WebsocketInfo = await api
		.get('api/websocket-info', {
			headers: {
				Authorization: token
			}
		})
		.then((res) => res.data);

	return { categories, WebsocketInfo };
};
