import assert from 'node:assert/strict';
import { afterEach, beforeEach, describe, it } from 'node:test';
import { cachedRequest, invalidateEtags } from './etag_cache.ts';

interface Call {
	url: string;
	method: string;
	ifNoneMatch: string | null;
	credentials: RequestCredentials | undefined;
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
			credentials: init?.credentials,
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
	it('includes session cookies for cross-origin login and authenticated reads', async () => {
		next = () => reply({ ok: true }, null);
		await cachedRequest(
			'https://api.example.test/api/auth/login',
			{ method: 'POST', body: '{}' },
			new Headers(),
		);
		await get('https://api.example.test/api/me');
		assert.deepEqual(calls.map((call) => call.credentials), ['include', 'include']);
	});

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

	it('hands back the body of a write so callers can read what they created', async () => {
		next = () => reply({ id: '01a10d41', title: 'Disastrous Necromancer' }, null);
		const created = await cachedRequest(
			'/api/works',
			{ method: 'POST', body: '{}' },
			new Headers(),
		);
		assert.deepEqual(created.body, { id: '01a10d41', title: 'Disastrous Necromancer' });
	});

	it('leaves the body empty for a write that has nothing to say', async () => {
		next = () => reply(undefined, null, 204);
		const empty = await cachedRequest('/api/library', { method: 'PUT', body: '{}' }, new Headers());
		assert.equal(empty.body, undefined);
	});

	it('leaves a failed write body unread so the caller can report why', async () => {
		next = () => reply({ error: 'nope' }, null, 422);
		const failed = await cachedRequest('/api/works', { method: 'POST', body: '{}' }, new Headers());
		assert.equal(failed.body, undefined);
		assert.deepEqual(await failed.response.json(), { error: 'nope' });
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

	it('keys paginated reads by their query string', async () => {
		next = () => reply({ entries: [{ id: 'w1' }], total: 120 }, '"v1"');
		const first = await get('/api/library?offset=0&limit=50');
		assert.deepEqual(first.body, { entries: [{ id: 'w1' }], total: 120 });

		next = () => reply({ entries: [{ id: 'w51' }], total: 120 }, '"v2"');
		const second = await get('/api/library?offset=50&limit=50');
		assert.equal(second.notModified, false);
		assert.deepEqual(second.body, { entries: [{ id: 'w51' }], total: 120 });
		assert.equal(calls[1].url, '/api/library?offset=50&limit=50');
		assert.equal(calls[1].ifNoneMatch, null);

		next = () => reply(null, '"v2"', 304);
		const replay = await get('/api/library?offset=50&limit=50');
		assert.equal(replay.notModified, true);
		assert.deepEqual(replay.body, { entries: [{ id: 'w51' }], total: 120 });
	});

	it('keys filtered reads separately from the unfiltered read', async () => {
		next = () => reply({ entries: [{ id: 'w1' }], total: 3 }, '"plain"');
		await get('/api/library?offset=0&limit=50');

		next = () => reply({ entries: [{ id: 'w2' }], total: 1 }, '"manga"');
		const filtered = await get('/api/library?offset=0&limit=50&kind=manga');
		assert.equal(filtered.notModified, false);
		assert.deepEqual(filtered.body, { entries: [{ id: 'w2' }], total: 1 });
		assert.equal(calls[1].ifNoneMatch, null);

		next = () => reply(null, '"manga"', 304);
		const replay = await get('/api/library?offset=0&limit=50&kind=manga');
		assert.equal(replay.notModified, true);
		assert.deepEqual(replay.body, { entries: [{ id: 'w2' }], total: 1 });

		next = () => reply(null, '"plain"', 304);
		const plain = await get('/api/library?offset=0&limit=50');
		assert.equal(plain.notModified, true);
		assert.deepEqual(plain.body, { entries: [{ id: 'w1' }], total: 3 });
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
