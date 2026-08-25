import { api, type PublicUser } from './api';

class AuthState {
	user = $state<PublicUser | null>(null);
	ready = $state(false);

	async init() {
		if (this.ready) return;
		try {
			this.user = await api.me();
		} catch {
			this.user = null;
		}
		this.ready = true;
	}

	async login(username: string, password: string) {
		await api.login(username, password);
		this.user = await api.me();
	}

	async register(username: string, password: string, inviteCode?: string) {
		await api.register(username, password, inviteCode);
		this.user = await api.me();
	}

	async logout() {
		try {
			await api.logout();
		} finally {
			this.user = null;
		}
	}
}

export const auth = new AuthState();
