<script lang="ts">
	import { onMount } from 'svelte';

	onMount(() => {
		const locationText = document.getElementById('LocationText');

		if (locationText) {
			locationText.innerText = 'Search';
		}
	});
</script>

<h1>Search</h1>
