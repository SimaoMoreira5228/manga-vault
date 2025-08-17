import { gql } from '@urql/svelte';
import { client } from './graphql/client';

type AuthState =
	| { status: 'loading' }
	| { status: 'unauthorized' }
	| { status: 'authenticated'; user: { id: number; username: string; imageId: number } };

let authState = $state<AuthState>({ status: 'loading' });

export function getAuthState() {
	return authState;
}

export function currentUser() {
	if (authState.status === 'authenticated') {
		return authState.user;
	}

	return null;
}

export async function initAuth() {
	try {
		const { data } = await client.query(
			gql`
				query Me {
					users {
						me {
							id
							username
							imageId
						}
					}
				}
			`,
			{}
		);

		if (!data.users.me) {
			authState = { status: 'unauthorized' };
			return;
		}

		authState = { status: 'authenticated', user: data.users.me };
	} catch {
		clearAuth();
	}
}

export async function login(input: { username: string; password: string }) {
	const result = await client
		.mutation(
			gql`
				mutation Login($input: LoginInput!) {
					auth {
						login(input: $input) {
							id
							username
							imageId
						}
					}
				}
			`,
			{ input }
		)
		.toPromise();

	if (result.error) {
		throw new Error(result.error.message.replace('[GraphQL] ', ''));
	}

	if (result.data?.auth.login) {
		authState = { status: 'authenticated', user: result.data.auth.login };
	}
}

// TODO: check if this works
export async function register(input: { username: string; password: string }) {
	const result = await client
		.mutation(
			gql`
				mutation Register($input: RegisterInput!) {
					auth {
						register(input: $input) {
							id
							username
							imageId
						}
					}
				}
			`,
			{ input }
		)
		.toPromise();

	if (result.data?.auth.register) {
		authState = { status: 'authenticated', user: result.data.auth.register };
	}
}

export async function logout() {
	await client
		.mutation(
			gql`
				mutation Logout {
					auth {
						logout
					}
				}
			`,
			{}
		)
		.toPromise();

	clearAuth();
}

function clearAuth() {
	authState = { status: 'unauthorized' };
}

initAuth();
