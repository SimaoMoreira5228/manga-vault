<script lang="ts">
	import * as Avatar from '$lib/components/ui/avatar/index';
	import { PlusIcon } from 'lucide-svelte';
	import type { PageData } from './$types';
	import { smallName } from '$lib/utils';

	export let data: PageData;

	const users = data.users;
</script>

<div class="flex h-full w-full items-center justify-center gap-8">
	{#each users as user}
		<a href="/users/login/{user.id}">
			<div class="flex flex-col items-center justify-center">
				<Avatar.Root class="h-32 w-32 rounded-lg">
					<Avatar.Image src={`/image/${user.image_id}`} alt="" class="object-cover" />
					<Avatar.Fallback class="h-32 w-32 rounded-lg">{smallName(user.username)}</Avatar.Fallback>
				</Avatar.Root>
				<p>{user.username}</p>
			</div>
		</a>
	{/each}
	<a href="/users/new" class="flex flex-col items-center justify-center">
		<div
			class="flex flex-col items-center justify-center rounded-lg border border-dashed border-primary"
		>
			<PlusIcon class="h-32 w-32 text-primary" />
		</div>
		<p class="text-gray-500">New User</p>
	</a>
</div>
