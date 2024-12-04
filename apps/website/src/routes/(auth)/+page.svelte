<script lang="ts">
	import { createStyles, Flex, Text, Skeleton } from '@svelteuidev/core';
	import { Plus } from 'radix-icons-svelte';
	import type { User } from '$lib/types';
	import { api, fileUrl } from '$lib/utils';
	import { themeStore as themeStoreClass } from '$lib';

	const users = api.get<User[]>('auth/users');
	const themeStore = themeStoreClass.store;

	const useStyles = createStyles(() => {
		return {
			root: {},
			avatarDiv: {
				display: 'flex',
				flexDirection: 'column',
				justifyContent: 'center',
				alignItems: 'center',
				gap: '0.5rem',
				textDecoration: 'none'
			},
			avatarImg: {
				width: '7rem',
				height: '7rem',
				borderRadius: '50%',
				objectFit: 'cover'
			},
			newUser: {
				display: 'flex',
				flexDirection: 'column',
				justifyContent: 'center',
				alignItems: 'center',
				width: '7rem',
				height: '7rem',
				borderRadius: '50%',
				border:
					$themeStore === 'dark'
						? '1px dashed var(--svelteui-colors-gray600)'
						: '1px dashed var(--svelteui-colors-dark400)'
			}
		};
	});

	$: ({ classes } = useStyles());
</script>

<Flex justify="center" gap="sm" wrap="wrap">
	{#await users then users}
		{#each users as user}
			<a class={classes.avatarDiv} href="/login/{user.id}">
				{#if user.image_id}
					<img src={fileUrl(user.image_id)} alt={user.username} class={classes.avatarImg} />
				{:else}
					<Skeleton height={112} width={112} circle />
				{/if}
				<Text>{user.username}</Text>
			</a>
		{/each}
	{/await}
	<a class={classes.avatarDiv} href="/new">
		<div class={classes.newUser}>
			<Plus
				color={$themeStore === 'dark'
					? 'var(--svelteui-colors-gray600)'
					: 'var(--svelteui-colors-dark400)'}
				size={24}
			/>
		</div>
		<Text>New User</Text>
	</a>
</Flex>
