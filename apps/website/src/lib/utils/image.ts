export function image(url: string): string {
	return `${import.meta.env.VITE_IMAGE_PROXY_URL}?url=${encodeURIComponent(url)}`;
}
