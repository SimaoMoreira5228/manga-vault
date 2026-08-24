export type WorkKind = 'manga' | 'novel';
export type ChapterContentKind = 'images' | 'html';

export interface Work {
	id: string;
	kind: WorkKind;
	source_id: string;
	remote_url: string;
	title: string;
	cover_url: string | null;
	alternative_names: string[];
	authors: string[];
	artists: string[];
	status: string | null;
	release_date: string | null;
	description: string | null;
	genres: string[];
	created_at: string;
	updated_at: string;
}

export interface Chapter {
	id: string;
	work_id: string;
	title: string;
	remote_url: string;
	sort_index: number;
	content_kind: ChapterContentKind;
	scanlation_group: string | null;
	released_at: string | null;
	created_at: string;
}

export interface RemoteWorkSummary {
	title: string;
	remote_url: string;
	cover_url: string | null;
}

export interface SourceInfo {
	id: string;
	name: string;
	version: string;
	kind: WorkKind;
	icon_url: string | null;
	referer_url: string | null;
	base_url: string | null;
}

export interface PublicUser {
	id: string;
	username: string;
	created_at: string;
}

export interface Session {
	token: string;
	user_id: string;
	device_label: string | null;
	created_at: string;
	last_seen_at: string;
}

export interface LibraryEntry {
	id: string;
	user_id: string;
	work_id: string;
	category_id: string | null;
	created_at: string;
}

export interface Category {
	id: string;
	user_id: string;
	name: string;
	created_at: string;
}

export interface ContinueReadingItem {
	work: Work;
	last_read: Chapter;
	next_chapter: Chapter | null;
	chapters_read: number;
	chapters_total: number;
}

export type ChapterContent = { Images: string[] } | { Html: string };

export type PluginBackend = 'lua' | 'wasm';

export interface StoredRepo {
	id: string;
	name: string;
	url: string;
}

export interface CatalogEntry {
	id: string;
	backend: PluginBackend;
	repo_id: string;
	repo_name: string;
	available_version: string;
	installed_version: string | null;
	update_available: boolean;
}

export type RegistrationMode = 'open' | 'closed' | 'invite';

export interface TrackerInfo {
	id: string;
	auth: 'paste' | 'oauth' | 'credentials';
}

export interface TrackerAccount {
	user_id: string;
	tracker_id: string;
	account_label: string | null;
}

export interface WorkTrackLink {
	id: string;
	work_id: string;
	tracker_id: string;
	remote_id: string;
	remote_title: string;
	remote_status: string | null;
	score: number | null;
	last_chapters_synced: number | null;
}

export interface GlossaryMeaning {
	id: string;
	meaning: string;
	votes: number;
	voted_by_me: boolean;
}

export interface GlossaryEntry {
	id: string;
	term: string;
	language: string;
	romanization: string | null;
	meanings: GlossaryMeaning[];
}

export interface InviteInfo {
	code: string;
	created_by: string;
	created_at: string;
	used_by: string | null;
}

export class ApiError extends Error {
	constructor(
		public status: number,
		message: string,
	) {
		super(message);
	}
}
export const API_BASE = import.meta.env.PUBLIC_API_URL ?? '';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const response = await fetch(`${API_BASE}${path}`, {
		credentials: 'include',
		...init,
		headers: { 'content-type': 'application/json', ...init?.headers },
	});
	if (!response.ok) {
		const body = await response.json().catch(() => ({ error: response.statusText }));
		throw new ApiError(response.status, body.error ?? 'request failed');
	}
	if (response.status === 204) return undefined as T;
	return response.json();
}

function get<T>(path: string) {
	return request<T>(path);
}
function post<T>(path: string, body?: unknown) {
	return request<T>(path, {
		method: 'POST',
		body: body === undefined ? undefined : JSON.stringify(body),
	});
}
function put<T>(path: string, body?: unknown) {
	return request<T>(path, {
		method: 'PUT',
		body: body === undefined ? undefined : JSON.stringify(body),
	});
}
function del<T>(path: string) {
	return request<T>(path, { method: 'DELETE' });
}

export function proxied(url: string): string {
	return `/api/proxy?url=${encodeURIComponent(url)}`;
}

export const api = {
	register: (username: string, password: string, inviteCode?: string) =>
		post<Session>('/api/auth/register', {
			username,
			password,
			...(inviteCode ? { invite_code: inviteCode } : {}),
		}),
	login: (username: string, password: string) =>
		post<Session>('/api/auth/login', { username, password }),
	logout: () => post('/api/auth/logout'),
	me: () => get<PublicUser>('/api/me'),
	sessions: () => get<Session[]>('/api/me/sessions'),
	revokeSession: (token: string) => del(`/api/me/sessions/${token}`),

	sources: () => get<SourceInfo[]>('/api/sources'),
	searchSource: (sourceId: string, query: string, page = 1) =>
		get<RemoteWorkSummary[]>(
			`/api/sources/${sourceId}/search?q=${encodeURIComponent(query)}&page=${page}`,
		),
	latestFromSource: (sourceId: string, page = 1) =>
		get<RemoteWorkSummary[]>(`/api/sources/${sourceId}/latest?page=${page}`),
	trendingFromSource: (sourceId: string, page = 1) =>
		get<RemoteWorkSummary[]>(`/api/sources/${sourceId}/trending?page=${page}`),

	importWork: (sourceId: string, remoteUrl: string) =>
		post<Work>('/api/works', { source_id: sourceId, remote_url: remoteUrl }),
	getWork: (workId: string) =>
		get<{ work: Work; chapters: Chapter[]; read_chapter_ids: string[] }>(`/api/works/${workId}`),
	requestRefresh: (workId: string) => post(`/api/works/${workId}/refresh`),
	chapterContent: (chapterId: string) => get<ChapterContent>(`/api/chapters/${chapterId}`),

	translationMode: () =>
		get<{ translation: { mode: string } }>('/api/me/capabilities').then((r) => r.translation.mode),
	saveTranslationSettings: (payload: { api_key: string; base_url?: string; model?: string }) =>
		put('/api/me/translation-settings', payload),
	clearTranslationSettings: () => del('/api/me/translation-settings'),
	translateChapter: (chapterId: string, to: string, from?: string) =>
		post<{ content: string; cached: boolean; matches: GlossaryEntry[] }>(
			`/api/chapters/${chapterId}/translate`,
			{ to, ...(from ? { from } : {}) },
		),

	trackersRegistry: () => get<{ trackers: TrackerInfo[] }>('/api/trackers'),
	myTrackerAccounts: () => get<TrackerAccount[]>('/api/me/trackers'),
	linkTracker: (
		id: string,
		payload: { token?: string; username?: string; password?: string },
	) => put(`/api/me/trackers/${id}`, payload),
	unlinkTracker: (id: string) => del(`/api/me/trackers/${id}`),
	startTrackerOauth: (id: string, redirectUri: string) =>
		post<{ authorize_url: string }>(`/api/me/trackers/${id}/oauth/start`, {
			redirect_uri: redirectUri,
		}),

	workTracks: (workId: string) => get<WorkTrackLink[]>(`/api/works/${workId}/track`),
	bindWorkTrack: (workId: string, trackerId: string, remoteId: string) =>
		post<WorkTrackLink>(`/api/works/${workId}/track`, {
			tracker_id: trackerId,
			remote_id: remoteId,
		}),
	deleteWorkTrack: (workId: string, linkId: string) => del(`/api/works/${workId}/track/${linkId}`),
	refreshWorkTrack: (workId: string, linkId: string) =>
		put<WorkTrackLink>(`/api/works/${workId}/track/${linkId}`, null),

	glossaryForLanguage: (language: string) => get<GlossaryEntry[]>(`/api/glossary?lang=${language}`),
	createGlossaryEntry: (payload: {
		term: string;
		language: string;
		meaning: string;
		romanization?: string;
	}) => post<GlossaryEntry>('/api/glossary', payload),
	addGlossaryMeaning: (entryId: string, meaning: string) =>
		post<GlossaryMeaning>(`/api/glossary/${entryId}/meanings`, { meaning }),
	toggleGlossaryVote: (meaningId: string) =>
		put<{ voted: boolean }>(`/api/glossary/meanings/${meaningId}/vote`, null),

	markRead: (chapterId: string) => put<{ id: string }>(`/api/chapters/${chapterId}/read`),
	markUnread: (chapterId: string) => del(`/api/chapters/${chapterId}/read`),

	continueReading: () => get<ContinueReadingItem[]>('/api/me/continue-reading'),

	library: () => get<{ entries: [LibraryEntry, Work][]; categories: Category[] }>('/api/library'),
	addToLibrary: (workId: string) => put<LibraryEntry>('/api/library', { work_id: workId }),
	removeFromLibrary: (workId: string) => del(`/api/library/${workId}`),

	pluginRepos: () => get<StoredRepo[]>('/api/plugin-repos'),
	addPluginRepo: (url: string) => post<StoredRepo>('/api/plugin-repos', { url }),
	removePluginRepo: (repoId: string) => del(`/api/plugin-repos/${repoId}`),
	pluginCatalog: () => get<CatalogEntry[]>('/api/plugins/catalog'),
	installPlugin: (pluginId: string, repoId?: string) =>
		put<unknown>(`/api/plugins/${pluginId}/install`, repoId ? { repo_id: repoId } : null),
	uninstallPlugin: (pluginId: string) => del(`/api/plugins/${pluginId}`),

	registrationMode: () => get<{ mode: RegistrationMode }>('/api/registration').then((r) => r.mode),
	setRegistrationMode: (mode: RegistrationMode) =>
		put<{ mode: RegistrationMode }>('/api/registration', { mode }),
	registrationAdminView: () =>
		get<{ mode: RegistrationMode; invites: InviteInfo[] }>('/api/registration/invites'),
	createInvite: () => post<InviteInfo>('/api/registration/invites'),
	deleteInvite: (code: string) => del(`/api/registration/invites/${code}`),
};
