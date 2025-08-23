import { env } from '$env/dynamic/public';

let proxyUrl = env.PUBLIC_IMAGE_PROXY_URL;
if (proxyUrl.endsWith('/')) {
	proxyUrl = proxyUrl.slice(0, -1);
}

let apiUrl = env.PUBLIC_API_URL;
if (!apiUrl.endsWith('/')) {
	apiUrl += '/';
}

export function proxyImage(url: string, referer?: string): string {
	if (referer) {
		return `${proxyUrl}?url=${encodeURIComponent(url)}&referer=${encodeURIComponent(referer)}`;
	} else {
		return `${proxyUrl}?url=${encodeURIComponent(url)}`;
	}
}

export function getImage(id: number): string {
	return `${apiUrl}files/${id}`;
}
