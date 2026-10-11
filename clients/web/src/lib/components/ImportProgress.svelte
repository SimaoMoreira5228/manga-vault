<script lang="ts">
let {
	title = 'Importing chapters',
	note = 'This can take a minute on a long series.',
}: { title?: string; note?: string } = $props();

let dialog = $state<HTMLDivElement | undefined>();

$effect(() => {
	dialog?.focus();
});
</script>

<div
	bind:this={dialog}
	class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-surface/85 px-6 text-center backdrop-blur-sm"
	role="dialog"
	aria-modal="true"
	aria-busy="true"
	aria-label={title}
	tabindex="-1"
>
	<svg class="size-8 animate-spin text-primary" viewBox="0 0 24 24" aria-hidden="true">
		<circle
			class="opacity-20"
			cx="12"
			cy="12"
			r="10"
			stroke="currentColor"
			stroke-width="4"
			fill="none"
		/>
		<path
			d="M4 12a8 8 0 0 1 8-8"
			stroke="currentColor"
			stroke-width="4"
			fill="none"
			stroke-linecap="round"
		/>
	</svg>
	<p class="title-md text-on-surface">{title}</p>
	<p class="body-md max-w-sm text-on-surface-variant">{note}</p>
</div>
