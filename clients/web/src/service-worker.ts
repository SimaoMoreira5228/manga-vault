/// <reference types="@sveltejs/kit" />
/// <reference lib="webworker" />

import { build, files, version } from '$service-worker';

const shellCache = `manga-vault-shell-${version}`;
const assetCache = `manga-vault-assets-${version}`;

const shell = [...build, ...files];

self.addEventListener('install', (event) => {
	event.waitUntil(
		caches
			.open(shellCache)
			.then((cache) => cache.addAll(shell))
			.then(() => self.skipWaiting()),
	);
});

self.addEventListener('activate', (event) => {
	event.waitUntil(
		caches
			.keys()
			.then((keys) =>
				Promise.all(
					keys
						.filter((key) => key !== shellCache && key !== assetCache)
						.map((key) => caches.delete(key)),
				),
			)
			.then(() => self.clients.claim()),
	);
});

self.addEventListener('fetch', (event) => {
	const request = event.request;
	if (request.method !== 'GET') return;

	const url = new URL(request.url);
	if (url.origin !== self.location.origin) return;
	// Authenticated API traffic must never be stored here: it is per-user, and the client
	// already revalidates it with ETags.
	if (url.pathname.startsWith('/api/')) return;

	if (request.mode === 'navigate') {
		event.respondWith(navigationResponse(request));
		return;
	}

	event.respondWith(assetResponse(request));
});

async function navigationResponse(request) {
	try {
		return await fetch(request);
	} catch {
		const cached = await caches.match('/index.html', { cacheName: shellCache });
		return cached ?? Response.error();
	}
}

async function assetResponse(request) {
	const cached = await caches.match(request, { cacheName: assetCache });
	if (cached) return cached;
	const response = await fetch(request);
	if (response.ok && response.type === 'basic') {
		const cache = await caches.open(assetCache);
		cache.put(request, response.clone());
	}
	return response;
}
