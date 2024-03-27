<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import Input from '$lib/components/ui/input/input.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let username = '';
	let password = '';

	onMount(() => {
		const usernameEl = document.getElementById('username') as HTMLInputElement;
		const passwordEl = document.getElementById('password') as HTMLInputElement;
		const confirmPasswordEl = document.getElementById('confirmPassword') as HTMLInputElement;

		confirmPasswordEl.addEventListener('input', () => {
			if (passwordEl.value !== confirmPasswordEl.value) {
				confirmPasswordEl.setCustomValidity('Passwords do not match');
			} else {
				confirmPasswordEl.setCustomValidity('');
			}
		});

		passwordEl.addEventListener('input', () => {
			if (passwordEl.value !== confirmPasswordEl.value) {
				confirmPasswordEl.setCustomValidity('Passwords do not match');
			} else {
				confirmPasswordEl.setCustomValidity('');
			}

			password = passwordEl.value;
		});

		usernameEl.addEventListener('input', () => {
			username = usernameEl.value;
		});
	});

	async function handleSubmit(event: Event) {
		event.preventDefault();

		const resp = await fetch('/users/new', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ username, password })
		});

		if (resp.ok) {
			goto('/');
		}
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	<Card.Root>
		<Card.Header>
			<Card.Title>Create new user</Card.Title>
			<Card.Description>Fill in the form to create a new user</Card.Description>
		</Card.Header>
		<Card.Content>
			<form method="post" class="flex flex-col items-center justify-center gap-4">
				<Input id="username" name="username" placeholder="Username" />
				<Input id="password" name="password" placeholder="Password" type="password" />
				<Input id="confirmPassword" placeholder="Password" type="password" />
				<Button on:click={handleSubmit}>Create user</Button>
			</form>
		</Card.Content>
		<Card.Footer>
			<p>Already have an account? <a href="/" class="text-blue-500">Login</a></p>
		</Card.Footer>
	</Card.Root>
</div>
