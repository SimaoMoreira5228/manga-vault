<script lang="ts">
let { value, max }: { value: number; max: number } = $props();

const fraction = $derived(max > 0 ? Math.min(1, value / max) : 0);
</script>

<div
	class="h-1 w-full bg-surface-highest"
	role="progressbar"
	aria-valuenow={value}
	aria-valuemax={max}
>
	<div class="h-full bg-primary-container" style={`width: ${fraction * 100}%`}></div>
</div>
