import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import {
	GO_SHORTCUTS,
	HELP_KEY,
	isTypingTarget,
	resolveShortcut,
	SEARCH_INPUT_ID,
	SEARCH_KEY,
	SEARCH_PATH,
	type ShortcutKeyEvent,
	shouldIgnoreKey,
} from './keyboard_shortcuts.ts';

function typing(overrides: Partial<ShortcutKeyEvent> = {}): ShortcutKeyEvent {
	return { key: 'a', ctrlKey: false, metaKey: false, altKey: false, target: null, ...overrides };
}

function element(tag: string, insideEditable = false): EventTarget {
	const closest = insideEditable ? () => ({}) : () => null;
	return { tagName: tag, closest } as unknown as EventTarget;
}

describe('shortcut table', () => {
	it('routes every g-sequence to the requested page', () => {
		const expected: Record<string, string> = {
			l: '/library',
			u: '/updates',
			h: '/history',
			s: '/stats',
			e: '/settings',
			x: SEARCH_PATH,
			d: '/',
		};
		for (const shortcut of GO_SHORTCUTS) {
			assert.equal(shortcut.href, expected[shortcut.key], `g ${shortcut.key}`);
		}
		assert.equal(GO_SHORTCUTS.length, Object.keys(expected).length);
	});

	it('keeps sequence keys unique and single lowercase letters', () => {
		const keys = GO_SHORTCUTS.map((shortcut) => shortcut.key);
		assert.equal(new Set(keys).size, keys.length);
		for (const key of keys) assert.match(key, /^[a-z]$/);
	});
});

describe('resolveShortcut', () => {
	it('sends / to search and asks for focus', () => {
		assert.deepEqual(resolveShortcut(SEARCH_KEY, false), {
			type: 'navigate',
			href: SEARCH_PATH,
			focusSearch: true,
		});
	});

	it('arms the sequence on g without navigating', () => {
		assert.deepEqual(resolveShortcut('g', false), { type: 'pending' });
	});

	it('completes a sequence with its route', () => {
		assert.deepEqual(resolveShortcut('l', true), { type: 'navigate', href: '/library' });
		assert.deepEqual(resolveShortcut('d', true), { type: 'navigate', href: '/' });
	});

	it('abandons the sequence on an unmapped second key', () => {
		assert.deepEqual(resolveShortcut('z', true), { type: 'idle' });
		assert.deepEqual(resolveShortcut('/', true), { type: 'idle' });
		assert.deepEqual(resolveShortcut('g', true), { type: 'idle' });
	});

	it('never treats the help key as navigation', () => {
		assert.deepEqual(resolveShortcut(HELP_KEY, false), { type: 'idle' });
	});

	it('ignores every other key', () => {
		for (const key of ['a', 'G', 'Enter', 'ArrowLeft', ' ']) {
			assert.deepEqual(resolveShortcut(key, false), { type: 'idle' }, key);
		}
	});

	it('exposes the id the search page binds its input to', () => {
		assert.equal(SEARCH_INPUT_ID, 'search-query');
	});
});

describe('isTypingTarget', () => {
	it('detects fields and editable regions', () => {
		for (const tag of ['INPUT', 'TEXTAREA', 'SELECT']) {
			assert.equal(isTypingTarget(element(tag)), true, tag);
		}
		assert.equal(isTypingTarget(element('DIV', true)), true);
		assert.equal(isTypingTarget(element('SPAN')), false);
	});

	it('does not treat plain elements or non-elements as typing', () => {
		assert.equal(isTypingTarget(element('DIV')), false);
		assert.equal(isTypingTarget(element('BUTTON')), false);
		assert.equal(isTypingTarget(null), false);
		assert.equal(isTypingTarget(globalThis), false);
	});
});

describe('shouldIgnoreKey', () => {
	it('passes plain keys through', () => {
		assert.equal(shouldIgnoreKey(typing({ key: SEARCH_KEY })), false);
		assert.equal(shouldIgnoreKey(typing({ key: HELP_KEY })), false);
		assert.equal(shouldIgnoreKey(typing()), false);
	});

	it('backs off for any modifier, since browsers and inputs own those combos', () => {
		assert.equal(shouldIgnoreKey(typing({ ctrlKey: true })), true);
		assert.equal(shouldIgnoreKey(typing({ metaKey: true })), true);
		assert.equal(shouldIgnoreKey(typing({ altKey: true })), true);
		assert.equal(shouldIgnoreKey(typing({ key: 'l', metaKey: true })), true);
	});

	it('backs off while the user types, including with shift held', () => {
		for (const tag of ['INPUT', 'TEXTAREA', 'SELECT']) {
			assert.equal(shouldIgnoreKey(typing({ target: element(tag) })), true, tag);
		}
		assert.equal(shouldIgnoreKey(typing({ key: '?', target: element('INPUT') })), true);
	});
});
