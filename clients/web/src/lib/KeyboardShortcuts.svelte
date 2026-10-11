<script lang="ts">
import { goto } from '$app/navigation';
import { page } from '$app/state';
import ShortcutHelp from '$lib/components/ShortcutHelp.svelte';
import {
	HELP_KEY,
	resolveShortcut,
	SEARCH_INPUT_ID,
	SEQUENCE_TIMEOUT_MS,
	shouldIgnoreKey,
} from '$lib/keyboard_shortcuts';

let helpOpen = $state(false);
let pending = false;
let sequenceTimer: ReturnType<typeof setTimeout> | null = null;

const shortcutsEnabled = $derived(!page.url.pathname.startsWith('/reader/'));

function resetSequence() {
	pending = false;
	if (sequenceTimer !== null) clearTimeout(sequenceTimer);
	sequenceTimer = null;
}

function armSequence() {
	pending = true;
	if (sequenceTimer !== null) clearTimeout(sequenceTimer);
	sequenceTimer = setTimeout(resetSequence, SEQUENCE_TIMEOUT_MS);
}

async function onKeydown(event: KeyboardEvent) {
	if (!shortcutsEnabled || shouldIgnoreKey(event)) return;

	if (event.key === HELP_KEY) {
		event.preventDefault();
		helpOpen = !helpOpen;
		return;
	}
	if (helpOpen) return;

	const action = resolveShortcut(event.key, pending);
	if (action.type === 'pending') {
		armSequence();
		return;
	}
	resetSequence();
	if (action.type !== 'navigate') return;

	event.preventDefault();
	await goto(action.href);
	if (action.focusSearch) document.getElementById(SEARCH_INPUT_ID)?.focus();
}

$effect(() => {
	window.addEventListener('keydown', onKeydown);
	return () => {
		window.removeEventListener('keydown', onKeydown);
		resetSequence();
	};
});
</script>

<ShortcutHelp open={helpOpen} onclose={() => (helpOpen = false)} />
