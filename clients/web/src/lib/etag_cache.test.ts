import assert from 'node:assert/strict';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { cachedRequest, invalidateEtags } from './etag_cache.ts';

interface Call {
	url: string;
	method: string;
	ifNoneMatch: string | null;
}

let calls: Call[] = [];
let next: () => Response;

function reply(body: unknown, etag: string | null, status = 200): Response {
	const headers = new Headers();
	if (etag !== null) headers.set('etag', etag);
	return new Response(status === 304 ? null : JSON.stringify(body), { status, headers });
}

beforeEach(() => {
	invalidateEtags();
	calls = [];
	next = () => reply({ stale: true }, null, 500);
	globalThis.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
		calls.push({
			url: String(input),
			method: init?.method ?? 'GET',
			ifNoneMatch: new Headers(init?.headers).get('if-none-match'),
		});
		return Promise.resolve(next());
	};
});

afterEach(() => {
	delete (globalThis as { fetch?: unknown }).fetch;
});

async function get(url: string) {
	return cachedRequest(url, undefined, new Headers());
}

describe('cachedRequest', () => {
	it('returns the cached body on 304 instead of parsing the empty body', async () => {
		next = () => reply({ name: 'dex' }, '"v1"');
		const first = await get('/api/sources');
		assert.equal(first.notModified, false);
		assert.deepEqual(first.body, { name: 'dex' });

		next = () => reply(null, '"v1"', 304);
		const second = await get('/api/sources');
		assert.equal(second.notModified, true);
		assert.deepEqual(second.body, { name: 'dex' });
		assert.equal(calls[1].ifNoneMatch, '"v1"');
	});

	it('replaces the cached body when the validator is stale', async () => {
		next = () => reply({ name: 'dex' }, '"v1"');
		await get('/api/sources');

		next = () => reply({ name: 'naruto' }, '"v2"');
		const second = await get('/api/sources');
		assert.equal(second.notModified, false);
		assert.deepEqual(second.body, { name: 'naruto' });

		next = () => reply(null, '"v2"', 304);
		const third = await get('/api/sources');
		assert.deepEqual(third.body, { name: 'naruto' });
	});

	it('invalidates every entry after a successful write', async () => {
		next = () => reply({ entries: [] }, '"v1"');
		await get('/api/library');

		next = () => reply({ ok: true }, null);
		await cachedRequest('/api/categories', { method: 'POST', body: '{}' }, new Headers());

		next = () => reply({ entries: [{ id: 'w1' }] }, '"v2"');
		const after = await get('/api/library');
		assert.equal(calls[2].ifNoneMatch, null);
		assert.deepEqual(after.body, { entries: [{ id: 'w1' }] });
	});

	it('never caches a response without an etag', async () => {
		next = () => reply({ name: 'dex' }, null);
		await get('/api/works/w1');
		await get('/api/works/w1');
		assert.equal(calls[0].ifNoneMatch, null);
		assert.equal(calls[1].ifNoneMatch, null);
	});

	it('never caches an error', async () => {
		next = () => reply({ error: 'boom' }, '"v1"', 500);
		await get('/api/library');
		await get('/api/library');
		assert.equal(calls[1].ifNoneMatch, null);
	});

	it('evicts the oldest entry past the bound', async () => {
		next = () => reply({ ok: true }, '"v1"');
		for (let index = 0; index < 40; index += 1) await get(`/api/works/${index}`);

		await get('/api/works/0');
		await get('/api/works/39');
		assert.equal(calls[40].ifNoneMatch, null);
		assert.equal(calls[41].ifNoneMatch, '"v1"');
	});
});
