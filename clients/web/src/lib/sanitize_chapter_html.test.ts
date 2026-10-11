import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';
import {
	ALLOWED_PROSE_ATTRS,
	ALLOWED_PROSE_TAGS,
	ALLOWED_URI_SCHEMES,
	sanitizeChapterHtml,
} from './sanitize_chapter_html.ts';

const serverSanitizer = readFileSync(
	fileURLToPath(new URL('../../../../crates/source-sdk/src/lib.rs', import.meta.url)),
	'utf8',
);

function rustAllowlist(marker: string): string[] {
	const markerAt = serverSanitizer.indexOf(marker);
	assert.notEqual(markerAt, -1, `${marker} is gone from sanitize_html`);
	const open = serverSanitizer.indexOf('[', markerAt);
	const close = serverSanitizer.indexOf(']', open);
	return [...serverSanitizer.slice(open, close).matchAll(/"([^"]+)"/g)].map((match) => match[1]);
}

describe('client sanitizer tracks the server allowlist', () => {
	it('permits exactly the tags ammonia permits', () => {
		assert.deepEqual([...ALLOWED_PROSE_TAGS].sort(), rustAllowlist('fn sanitize_html').sort());
	});

	it('permits exactly the attributes ammonia permits', () => {
		assert.deepEqual(ALLOWED_PROSE_ATTRS, rustAllowlist('.generic_attributes('));
	});

	it('permits exactly the url schemes ammonia permits', () => {
		const schemes = rustAllowlist('.url_schemes(');
		assert.deepEqual(schemes, ['http', 'https', 'mailto']);
		for (const scheme of schemes) {
			assert.ok(ALLOWED_URI_SCHEMES.test(`${scheme}:host`), `${scheme}: should be reachable`);
		}
	});

	it('rejects the schemes ammonia rejects', () => {
		for (const scheme of ['javascript', 'data', 'vbscript', 'file']) {
			assert.ok(!ALLOWED_URI_SCHEMES.test(`${scheme}:host`), `${scheme}: must not be reachable`);
		}
	});

	it('does not throw where dompurify has no dom, so ssr keeps rendering', () => {
		const raw = '<p>chapter text</p>';
		assert.equal(sanitizeChapterHtml(raw), raw);
	});
});
