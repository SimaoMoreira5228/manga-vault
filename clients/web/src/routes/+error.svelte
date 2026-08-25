<script lang="ts">
import { page } from '$app/state';
</script>

<div class="grid min-h-dvh place-items-center p-10 text-center">
	<div>
		<h1 class="font-display text-6xl font-bold text-primary">{page.status}</h1>
		<p class="body-lg mt-4 text-on-surface-variant">
			{page.error?.message ?? 'Something went wrong.'}
		</p>
		<a
			href="/"
			class="label-caps mt-8 inline-block rounded-card bg-primary-container px-6 py-3 font-semibold text-on-primary-container hover:opacity-90"
		>
			Back to Explore
		</a>
	</div>
</div>
