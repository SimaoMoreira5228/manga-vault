const maxEntries = 32;

interface CachedBody {
	etag: string;
	body: unknown;
}

const cache = new Map<string, CachedBody>();

export interface CachedResponse {
	response: Response;
	body: unknown;
	notModified: boolean;
}

function stash(url: string, etag: string, body: unknown): void {
	cache.delete(url);
	if (cache.size >= maxEntries) {
		const oldest = cache.keys().next().value;
		if (oldest !== undefined) cache.delete(oldest);
	}
	cache.set(url, { etag, body });
}

export function invalidateEtags(): void {
	cache.clear();
}

export async function cachedRequest(
	url: string,
	init: RequestInit | undefined,
	headers: Headers,
): Promise<CachedResponse> {
	if (init?.method !== undefined && init.method !== 'GET') {
		const response = await fetch(url, { ...init, headers });
		if (response.ok) invalidateEtags();
		if (!response.ok || response.status === 204) {
			return { response, body: undefined, notModified: false };
		}
		return { response, body: await response.json(), notModified: false };
	}

	const cached = cache.get(url);
	const conditional = new Headers(headers);
	if (cached) conditional.set('if-none-match', cached.etag);
	const response = await fetch(url, { ...init, headers: conditional });

	if (response.status === 304 && cached) {
		return { response, body: cached.body, notModified: true };
	}
	if (!response.ok || response.status === 204) {
		return { response, body: undefined, notModified: false };
	}

	const body: unknown = await response.json();
	const etag = response.headers.get('etag');
	if (etag !== null) stash(url, etag, body);
	return { response, body, notModified: false };
}
