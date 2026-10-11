import DOMPurify from 'dompurify';

export const ALLOWED_PROSE_TAGS = [
	'a',
	'b',
	'blockquote',
	'br',
	'code',
	'del',
	'em',
	'h1',
	'h2',
	'h3',
	'h4',
	'h5',
	'h6',
	'hr',
	'i',
	'ins',
	'li',
	'mark',
	'ol',
	'p',
	'pre',
	'q',
	's',
	'small',
	'strong',
	'sub',
	'sup',
	'u',
	'ul',
];

export const ALLOWED_PROSE_ATTRS = ['lang', 'title'];

export const ALLOWED_URI_SCHEMES = /^(?:https?:|mailto:)/i;

export function sanitizeChapterHtml(raw: string): string {
	// SECURITY: workerd and node expose no DOM, so DOMPurify is inert there and the
	// server's ammonia pass is the only gate. The browser pass is additive.
	if (typeof DOMPurify.sanitize !== 'function') return raw;
	return DOMPurify.sanitize(raw, {
		ALLOWED_TAGS: ALLOWED_PROSE_TAGS,
		ALLOWED_ATTR: ALLOWED_PROSE_ATTRS,
		ALLOWED_URI_REGEXP: ALLOWED_URI_SCHEMES,
	});
}
