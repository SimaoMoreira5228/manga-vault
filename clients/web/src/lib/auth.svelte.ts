import { api, type PublicUser, setUnauthorizedHandler } from './api';

class AuthState {
	user = $state<PublicUser | null>(null);
	ready = $state(false);
	private registered = false;

	get isAdmin() {
		return this.user?.is_admin ?? false;
	}

	async init() {
		if (!this.registered) {
			setUnauthorizedHandler(() => {
				this.user = null;
			});
			this.registered = true;
		}
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
