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
		<Avatar.Root class="bg-background h-full w-full rounded-full">
			{#if loadingAvatar}
				<div class="absolute inset-0 flex items-center justify-center bg-black bg-opacity-10">
					<Spinner class="text-primary h-12 w-12" />
				</div>
			{:else}
				<Avatar.Image src={`/image/${image_id}`} alt="" class="h-full w-full object-cover" />
				<Avatar.Fallback class="bg-input h-full w-full rounded-lg">
					{smallName(data.user.username)}
				</Avatar.Fallback>
			{/if}
		</Avatar.Root>
		<Label for="picture" class="bg-secondary absolute bottom-5 right-5 z-20 h-6 w-6 rounded">
			<ImagePlusIcon class="text-primary h-6 w-6" />
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
