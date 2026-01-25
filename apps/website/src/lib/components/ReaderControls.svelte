<script lang="ts">
import { Slider, Switch } from "@skeletonlabs/skeleton-svelte";

interface Props {
	isNovel?: boolean;
	imageMargin?: number;
	brSpacing?: number;
	autoNext?: boolean;
	onImageMarginChange?: (value: number) => void;
	onBrSpacingChange?: (value: number) => void;
	onAutoNextChange?: (value: boolean) => void;
}

let {
	isNovel = false,
	imageMargin = 0,
	brSpacing = 8,
	autoNext = false,
	onImageMarginChange,
	onBrSpacingChange,
	onAutoNextChange,
}: Props = $props();
</script>

<div class="flex w-full items-center gap-4">
	{#if !isNovel}
		<label class="label w-9/10 flex items-center gap-2">
			<span class="label-text">Image Margin: {imageMargin}%</span>
			<Slider
				name="image-margin"
				value={[imageMargin]}
				onValueChange={(e) => onImageMarginChange?.(e.value[0])}
				min={0}
				max={45}
			/>
		</label>
	{/if}

	{#if isNovel}
		<label class="label w-9/10 flex items-center gap-2">
			<span class="label-text">BR Spacing: </span>
			<input
				type="number"
				min="0"
				max="100"
				value={brSpacing}
				oninput={(e) => onBrSpacingChange?.(parseInt((e.target as HTMLInputElement).value || "0", 10) || 0)}
				class="input w-32"
			/>
		</label>
	{/if}

	<label class="label w-1/10 flex items-center gap-2">
		<span class="label-text">Auto-next</span>
		<Switch
			name="auto-next"
			checked={autoNext}
			onCheckedChange={(e) => onAutoNextChange?.(e.checked)}
		/>
	</label>
</div>
