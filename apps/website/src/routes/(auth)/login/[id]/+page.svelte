<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import type { User } from '$lib/types';
	import { api, fileUrl } from '$lib/utils';
	import {
		Flex,
		Card,
		createStyles,
		Group,
		Title,
		PasswordInput,
		Button,
		Skeleton
	} from '@svelteuidev/core';
	import { ArrowLeft } from 'radix-icons-svelte';
	import { toast } from 'svelte-sonner';

	const userId = $page.params.id;

	const user = api.get<User>(`auth/users/${userId}`);

	const useStyles = createStyles(() => {
		return {
			root: {},
			avatarImg: {
				width: '7rem',
				height: '7rem',
				borderRadius: '50%',
				objectFit: 'cover'
			},
			form: {
				display: 'flex',
				flexDirection: 'column',
				gap: '0.5rem',
				mt: '1.5rem',
				width: '24rem',
				'@media (max-width: 768px)': {
					width: '18rem'
				}
			}
		};
	});

	$: ({ classes } = useStyles());

	let password: string;

	const handleSubmit = async () => {
		try {
			await api.post(`auth/login`, {
				username: (await user).username,
				password
			});
			goto('/library');
		} catch (e: any) {
			toast.error(e.message as string);
		}
	};
</script>

<Flex justify="center">
	{#await user then user}
		<Card shadow="lg" padding="lg">
			<Button variant="subtle" on:click={() => goto('/')} style="padding: 4px;">
				<ArrowLeft size={24} />
			</Button>
			<Group position="apart" direction="column">
				{#if user.image_id}
					<img src={fileUrl(user.image_id)} alt="" class={classes.avatarImg} />
				{:else}
					<Skeleton height={112} width={112} circle />
				{/if}
				<Title order={1}>{user.username}</Title>
			</Group>
			<form class={classes.form} on:submit={handleSubmit}>
				<PasswordInput label="Password" bind:value={password} required class={classes.input} />
				<Button type="submit" variant="light" style="width: 100%;">Login</Button>
			</form>
		</Card>
	{/await}
</Flex>
