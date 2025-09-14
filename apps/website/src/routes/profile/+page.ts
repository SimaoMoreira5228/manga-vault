import { waitForAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import { gql } from "@urql/svelte";

export async function load() {
	const authState = await waitForAuthState();
	if (authState.status !== "authenticated") return;

	const result = await client
		.query(
			gql`
				query getUserFiles {
					files {
						files {
							id
						}
					}
				}
			`,
			{},
		)
		.toPromise();

	if (result.error) {
		console.error("Failed to fetch user files", result.error);
	}

	const filesArray = result.data?.files.files as { id: number }[];
	return { fileIds: filesArray.map((file) => file.id) || [] };
}
