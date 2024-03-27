import type { Cookies } from '@sveltejs/kit';
import { api } from './axios.server';

export type User = {
	id: number;
	username: string;
};

export async function getUser(cookies: Cookies) {
	const token = cookies.get('token');

	if (!token) {
		return null;
	}

	try {
		const resp = await api.get('api/me', { headers: { Authorization: token } });
		if (resp.status === 200) {
			const user = resp.data as User;
			return user;
		}
	} catch (_) {
		return null;
	}

	return null;
}
