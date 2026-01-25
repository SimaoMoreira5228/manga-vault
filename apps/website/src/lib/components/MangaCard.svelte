<script lang="ts">
import { proxyImage } from "$lib/utils/image";

interface Props {
	work: {
		title?: string;
		imgUrl?: string | null;
		chaptersAmount?: number | null;
		userReadChaptersAmount?: number | null;
	};
	href?: string | null;
	refererUrl?: string | null;
	showBadge?: boolean;
}

let {
	work,
	href = null,
	refererUrl = undefined,
	showBadge = true,
}: Props = $props();

function badgeCount() {
	const a = Number(work?.chaptersAmount ?? 0);
	const b = Number(work?.userReadChaptersAmount ?? 0);
	const v = a - b;
	return v > 0 ? v : 0;
}
</script>

{#if href}
	<a
		class="card relative flex h-80 w-full max-w-48 flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat shadow-lg"
		style="background-image: url({proxyImage(work?.imgUrl ?? '', refererUrl ?? undefined)});"
		href={href}
		rel="external"
	>
		<div class="absolute inset-0 bg-linear-to-b from-transparent to-black/75"></div>

		<div class="relative z-10 w-full truncate p-4 text-center text-base text-white" title={work?.title}>
			{work?.title}
		</div>

		{#if showBadge && badgeCount() > 0}
			<span class="badge-icon preset-filled-primary-500 absolute right-0 top-0 z-10">{badgeCount()}</span>
		{/if}
	</a>
{:else}
	<div
		class="card relative flex h-80 w-full max-w-48 flex-col items-start justify-end overflow-hidden rounded-lg bg-cover bg-center bg-no-repeat shadow-lg"
		style="background-image: url({proxyImage(work?.imgUrl ?? '', refererUrl ?? undefined)});"
	>
		<div class="absolute inset-0 bg-linear-to-b from-transparent to-black/75"></div>

		<div class="relative z-10 w-full truncate p-4 text-center text-base text-white" title={work?.title}>
			{work?.title}
		</div>

		{#if showBadge && badgeCount() > 0}
			<span class="badge-icon preset-filled-primary-500 absolute right-0 top-0 z-10">{badgeCount()}</span>
		{/if}
	</div>
{/if}
