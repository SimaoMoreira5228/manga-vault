export function proxyImage(url: string, referer?: string): string {
	if (referer) {
		return `${import.meta.env.VITE_IMAGE_PROXY_URL}?url=${encodeURIComponent(url)}&referer=${encodeURIComponent(referer)}`;
	} else {
		return `${import.meta.env.VITE_IMAGE_PROXY_URL}?url=${encodeURIComponent(url)}`;
	}
}

export function getImage(id: number): string {
	return `${import.meta.env.VITE_API_URL}files/${id}`;
}
