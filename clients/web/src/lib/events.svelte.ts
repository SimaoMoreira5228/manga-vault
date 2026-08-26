import { API_BASE } from '$lib/api';

type RefreshListener = (workId: string) => void;

const listeners = new Set<RefreshListener>();
let source: EventSource | null = null;

function ensureSource() {
	if (source || typeof window === 'undefined') return;
	source = new EventSource(`${API_BASE}/api/events`, { withCredentials: true });
	source.onmessage = (message) => {
		try {
			const event = JSON.parse(message.data) as { type?: string; work_id?: string };
			if (event.type !== 'work_refreshed' || !event.work_id) return;
			for (const listener of listeners) listener(event.work_id);
			const title = titleLookup?.(event.work_id);
			if (title) notifyChapters(title, 'New chapters available');
		} catch {
			// malformed payload: ignore
		}
	};
	source.onerror = () => {
		source?.close();
		source = null;
		setTimeout(ensureSource, 5000);
	};
}

export function onWorkRefreshed(listener: RefreshListener): () => void {
	if (typeof window === 'undefined') return () => {};
	ensureSource();
	listeners.add(listener);
	return () => listeners.delete(listener);
}

const NOTIFICATIONS_KEY = 'mv-chapter-notifications';

export function chapterNotificationsEnabled(): boolean {
	return typeof window !== 'undefined' && localStorage.getItem(NOTIFICATIONS_KEY) === 'on';
}

export async function setChapterNotifications(enabled: boolean): Promise<boolean> {
	if (!enabled) {
		localStorage.removeItem(NOTIFICATIONS_KEY);
		return true;
	}
	if (!('Notification' in window)) return false;
	const permission = await Notification.requestPermission();
	if (permission !== 'granted') return false;
	localStorage.setItem(NOTIFICATIONS_KEY, 'on');
	notifyChapters('Notifications enabled', 'You will hear about new chapters here.');
	return true;
}

function notifyChapters(title: string, body: string) {
	if (!chapterNotificationsEnabled() || !('Notification' in window)) return;
	if (Notification.permission !== 'granted') return;
	new Notification(title, { body, tag: title });
}

let titleLookup: ((workId: string) => string | undefined) | null = null;

export function registerWorkTitles(lookup: (workId: string) => string | undefined) {
	titleLookup = lookup;
}
