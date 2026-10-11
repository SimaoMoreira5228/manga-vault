<script lang="ts">
import { GO_SHORTCUTS, HELP_KEY, SEARCH_KEY, SEQUENCE_PREFIX } from '$lib/keyboard_shortcuts';

let { open, onclose }: { open: boolean; onclose: () => void } = $props();

let dialog = $state<HTMLDivElement | null>(null);
let previousFocus: HTMLElement | null = null;

const rows = [
	{ keys: [SEARCH_KEY], label: 'Focus search' },
	...GO_SHORTCUTS.map((shortcut) => ({
		keys: [SEQUENCE_PREFIX, shortcut.key],
		label: shortcut.label,
	})),
	{ keys: [HELP_KEY], label: 'Toggle this help' },
	{ keys: ['Esc'], label: 'Close' },
];

const focusables = 'a[href], button, [tabindex]:not([tabindex="-1"])';

$effect(() => {
	if (!open) return;
	previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
	dialog?.focus();
	return () => {
		previousFocus?.focus();
		previousFocus = null;
	};
});

function onKeydown(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		event.preventDefault();
		onclose();
		return;
	}
	if (event.key !== 'Tab' || dialog === null) return;
	const items = dialog.querySelectorAll<HTMLElement>(focusables);
	const first = items[0];
	const last = items[items.length - 1];
	if (event.shiftKey && document.activeElement === first) {
		event.preventDefault();
		last?.focus();
	} else if (!event.shiftKey && document.activeElement === last) {
		event.preventDefault();
		first?.focus();
	}
}
</script>

{#if open}
	<div
		bind:this={dialog}
		class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-4"
		role="dialog"
		aria-modal="true"
		aria-labelledby="shortcut-help-title"
		tabindex="-1"
		onclick={(event) => {
			if (event.target === event.currentTarget) onclose();
		}}
		onkeydown={onKeydown}
	>
		<div
			class="w-full max-w-sm rounded-card border border-outline-variant/40 bg-surface-low p-6 shadow-elevated"
		>
			<h2 id="shortcut-help-title" class="title-md">Keyboard shortcuts</h2>
			<dl class="mt-5 flex flex-col gap-2">
				{#each rows as row (row.keys.join(' '))}
					<div class="flex items-center justify-between gap-4">
						<dt class="mono-label text-on-surface-variant">{row.label}</dt>
						<dd class="flex shrink-0 items-center gap-1">
							{#each row.keys as key, index (key)}
								{#if index > 0}
									<span class="mono-label text-outline-variant">then</span>
								{/if}
								<kbd
									class="label-caps rounded-card border border-outline-variant/60 bg-surface-container px-2 py-1 text-on-surface"
									>{key}</kbd
								>
							{/each}
						</dd>
					</div>
				{/each}
			</dl>
			<button
				type="button"
				class="label-caps mt-6 w-full rounded-card border border-outline-variant/60 px-4 py-2 text-on-surface hover:border-outline"
				onclick={onclose}
			>
				Close
			</button>
		</div>
	</div>
{/if}
