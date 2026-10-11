export const SEQUENCE_PREFIX = 'g';
export const SEARCH_KEY = '/';
export const HELP_KEY = '?';
export const SEARCH_PATH = '/search';
export const SEARCH_INPUT_ID = 'search-query';
export const SEQUENCE_TIMEOUT_MS = 1000;

export interface GoShortcut {
	key: string;
	label: string;
	href: string;
}

export const GO_SHORTCUTS: readonly GoShortcut[] = [
	{ key: 'd', label: 'Explore', href: '/' },
	{ key: 'l', label: 'Library', href: '/library' },
	{ key: 'u', label: 'Updates', href: '/updates' },
	{ key: 'h', label: 'History', href: '/history' },
	{ key: 's', label: 'Stats', href: '/stats' },
	{ key: 'e', label: 'Settings', href: '/settings' },
	{ key: 'x', label: 'Search', href: SEARCH_PATH },
];

export type ShortcutAction =
	| { type: 'navigate'; href: string; focusSearch?: boolean }
	| { type: 'pending' }
	| { type: 'idle' };

const GO_HREF: Record<string, string> = Object.fromEntries(
	GO_SHORTCUTS.map((shortcut) => [shortcut.key, shortcut.href]),
);

export function resolveShortcut(key: string, pending: boolean): ShortcutAction {
	if (pending) {
		const href = GO_HREF[key];
		return href === undefined ? { type: 'idle' } : { type: 'navigate', href };
	}
	if (key === SEQUENCE_PREFIX) return { type: 'pending' };
	if (key === SEARCH_KEY) return { type: 'navigate', href: SEARCH_PATH, focusSearch: true };
	return { type: 'idle' };
}

const TYPING_TAGS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

export function isTypingTarget(target: EventTarget | null): boolean {
	const element = target as Element | null;
	if (element === null || typeof element?.tagName !== 'string') return false;
	if (TYPING_TAGS.has(element.tagName)) return true;
	return element.closest?.('[contenteditable]') != null;
}

export interface ShortcutKeyEvent {
	key: string;
	ctrlKey: boolean;
	metaKey: boolean;
	altKey: boolean;
	target: EventTarget | null;
}

export function shouldIgnoreKey(event: ShortcutKeyEvent): boolean {
	return event.ctrlKey || event.metaKey || event.altKey || isTypingTarget(event.target);
}
