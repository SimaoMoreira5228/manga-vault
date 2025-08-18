export function image(url: string, referer?: string): string {
	if (referer) {
		return `${import.meta.env.VITE_IMAGE_PROXY_URL}?url=${encodeURIComponent(url)}&referer=${encodeURIComponent(referer)}`;
	} else {
		return `${import.meta.env.VITE_IMAGE_PROXY_URL}?url=${encodeURIComponent(url)}`;
	}
}
