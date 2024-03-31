<script lang="ts">
	import * as Avatar from '$lib/components/ui/avatar';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { ImagePlusIcon } from 'lucide-svelte';
	import { smallName } from '$lib/utils';
	import type { PageData } from './$types';
	import { toast } from 'svelte-sonner';
	import Spinner from '$lib/icons/spinner.svelte';

	export let data: PageData;

	$: image_id = data.user.image_id;
	let loadingAvatar = false;

	async function handleFileChange(event: Event) {
		try {
			loadingAvatar = true;
			const input = event.target as HTMLInputElement;
			const img = input.files?.[0];

			if (!img) return;

			const formData = new FormData();
			formData.append('img', img);

			if (img) {
				const response = await fetch('/library/profile', {
					method: 'POST',
					body: formData
				});

				image_id = await response.json();

				if (response.ok) {
					const response = await fetch('/library/profile/image', {
						method: 'PATCH',
						headers: {
							'Content-Type': 'application/json'
						},
						body: JSON.stringify({
							user_id: data.user.id,
							image_id
						})
					});

					if (response.ok) {
						toast('Image uploaded');
					} else {
						toast(`Error uploading image: ${response.statusText}`);
					}
				} else {
					toast(`Error uploading image: ${response.statusText}`);
				}
			}
		} catch (error) {
			toast(`Error uploading image: ${error}`);
		} finally {
			loadingAvatar = false;
		}
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-start gap-4">
	<div class="relative h-64 w-64">
		<Avatar.Root class="h-full w-full rounded-full bg-background">
			{#if loadingAvatar}
				<div class="absolute inset-0 flex items-center justify-center bg-black bg-opacity-10">
					<Spinner class="h-12 w-12 text-primary" />
				</div>
			{:else}
				<Avatar.Image src={`/image/${image_id}`} alt="" class="h-full w-full object-cover" />
				<Avatar.Fallback class="h-full w-full rounded-lg bg-input">
					{smallName(data.user.username)}
				</Avatar.Fallback>
			{/if}
		</Avatar.Root>
		<Label for="picture" class="absolute bottom-5 right-5 z-20 h-6 w-6 rounded bg-secondary">
			<ImagePlusIcon class="h-6 w-6 text-primary" />
		</Label>
		<Input
			id="picture"
			type="file"
			class="sr-only cursor-pointer"
			accept="image/*"
			on:change={handleFileChange}
		/>
	</div>
	<h1 class="text-2xl">{data.user.username}</h1>
</div>
