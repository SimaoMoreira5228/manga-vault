<script lang="ts">
	import { goto } from '$app/navigation';
	import { ArrowLeft } from 'radix-icons-svelte';
	import type { User } from '$lib/types';
	import { api } from '$lib/utils';
	import {
		Button,
		Card,
		createStyles,
		Flex,
		PasswordInput,
		TextInput,
		Title
	} from '@svelteuidev/core';
	import { toast } from 'svelte-sonner';

	let username: string;
	let password: string;

	const useStyles = createStyles(() => {
		return {
			root: {},
			card: {
				display: 'flex',
				flexDirection: 'column',
				justifyContent: 'center',
				alignItems: 'center'
			},
			bar: {
				display: 'flex',
				flexDirection: 'row',
				justifyContent: 'start',
				alignItems: 'center',
				width: '100%'
			},
			tittle: {
				display: 'flex',
				flexDirection: 'row',
				justifyContent: 'center',
				alignItems: 'center',
				width: '100%'
			},
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

	$: ({ classes, getStyles } = useStyles());

	const handleSubmit = async () => {
		try {
			const res = await api.post<User>(`auth/users/create`, {
				username,
				password
			});
			goto(`/login/${res.id}`);
		} catch (e: any) {
			toast.error(e.message as string);
		}
	};
</script>

<Flex justify="center" class={getStyles()}>
	<Card shadow="lg" padding="lg" class={classes.card}>
		<div class={classes.bar}>
			<Button variant="subtle" on:click={() => goto('/')} style="padding: 4px;">
				<ArrowLeft size={24} />
			</Button>
			<div class={classes.tittle}>
				<Title order={1}>Create User</Title>
			</div>
		</div>
		<form class={classes.form} on:submit|preventDefault={handleSubmit}>
			<TextInput label="Username" bind:value={username} required class={classes.input} />
			<PasswordInput label="Password" bind:value={password} required class={classes.input} />
			<Button type="submit" variant="light" style="width: 100%;">Create Account</Button>
		</form>
	</Card>
</Flex>
