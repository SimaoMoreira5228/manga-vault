<script lang="ts">
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { getAuthState, updateAuth, waitForAuthState } from "$lib/auth.svelte";
import { client } from "$lib/graphql/client";
import DotsSpinner from "$lib/icons/DotsSpinner.svelte";
import { getImage } from "$lib/utils/image";
import { toaster } from "$lib/utils/toaster-svelte";
import { PenLine, Plus, UserRound } from "@lucide/svelte";
import { Modal } from "@skeletonlabs/skeleton-svelte";
import { gql } from "@urql/svelte";
import { Avatar } from "bits-ui";
import type { PageData } from "./$types";

type isUpdatingCategoryType = {
	open: boolean;
	input: {
		username: string | null;
		imageId: number | null;
	};
};

let { data }: { data: PageData } = $props();

let authState = $derived(getAuthState());
let fileIds = $derived.by(() => {
	if (data.fileIds) {
		return data.fileIds;
	}

	return [];
});
let fileInput: HTMLInputElement;
let isUploadingFile = $state(false);
let isUpdatingCategory = $state<isUpdatingCategoryType>({ open: false, input: { username: null, imageId: null } });

$effect(() => {
	if (authState.status === "loading") return;
	if (authState.status === "unauthorized") {
		goto(resolve("/login"));
	}
});

async function getUserFiles() {
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
	fileIds = filesArray.map((file) => file.id) || [];
}

async function uploadFile(file: File) {
	const authState = await waitForAuthState();
	if (authState.status !== "authenticated") return;
	isUploadingFile = true;

	try {
		await client
			.mutation(
				gql`
						mutation uploadProfileFile($file: Upload!) {
							files {
								uploadFile(file: $file) {
									id
								}
							}
						}
					`,
				{ file },
			)
			.toPromise();

		await getUserFiles();
	} catch (error) {
		console.error("Failed to upload file", error);
		toaster.error({ title: "Error", description: "Failed to upload file" });
	} finally {
		isUploadingFile = false;
	}
}

function handleInputChange(event: Event) {
	const input = event.target as HTMLInputElement;
	if (input.files?.[0]) {
		if (!input.files[0]) return;
		uploadFile(input.files[0]);
	}
}

async function updateProfile() {
	if (authState.status !== "authenticated") return;
	isUpdatingCategory.open = false;

	try {
		await client
			.mutation(
				gql`
						mutation updateProfile($input: UpdateProfileInput!) {
							profile {
								updateProfile(input: $input) {
									id
								}
							}
						}
					`,
				{ input: isUpdatingCategory.input },
			)
			.toPromise();

		await updateAuth();
	} catch (error) {
		console.error("Failed to update profile", error);
		toaster.error({ title: "Error", description: "Failed to update profile" });
	}
}

function openUpdateProfileModal() {
	if (authState.status !== "authenticated") return;

	isUpdatingCategory.input.username = authState.user.username;
	isUpdatingCategory.open = true;
}
</script>

{#if authState.status === "authenticated"}
	<div class="flex h-full w-full flex-col items-center justify-center">
		{#if authState.user.imageId}
			<Avatar.Root delayMs={200} class="h-48 w-48 rounded-full">
				<div class="flex h-full w-full items-center justify-center overflow-hidden rounded-full border-transparent">
					<Avatar.Image
						src={getImage(authState.user.imageId)}
						alt=""
						class="h-48 w-48 rounded-full object-cover"
					/>
					<Avatar.Fallback class="flex h-48 w-48 items-center justify-center rounded-full">
						<UserRound size={96} />
					</Avatar.Fallback>
				</div>
			</Avatar.Root>
		{:else}
			<Avatar.Root delayMs={200} class="h-48 w-48 rounded-full border">
				<div class="flex h-full w-full items-center justify-center overflow-hidden rounded-full border-2 border-transparent">
					<Avatar.Fallback class="flex h-48 w-48 items-center justify-center rounded-full">
						<UserRound size={96} />
					</Avatar.Fallback>
				</div>
			</Avatar.Root>
		{/if}
		<h5 class="h5 flex flex-row items-center gap-2">
			{authState.user.username}
			<button class="opacity-60" onclick={openUpdateProfileModal}>
				<PenLine size={16} />
			</button>
		</h5>
		<div class="h-6/10 flex flex-col items-center justify-center overflow-auto">
			<div class="mt-4 grid grid-cols-1 justify-items-center gap-4 overflow-auto p-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				<button
					class="border-primary-100-900 flex h-72 w-72 flex-col items-center justify-center rounded-lg border border-dashed"
					onclick={() => (isUploadingFile ? null : fileInput.click())}
				>
					{#if isUploadingFile}
						<DotsSpinner class="h-8 w-8" />
					{:else}
						<Plus />
					{/if}
				</button>
				{#each fileIds as fileId (fileId)}
					<button
						onclick={async () => {
							isUpdatingCategory.input.imageId = fileId;
							await updateProfile();
						}}
					>
						<img
							src={getImage(fileId)}
							alt=""
							class="
								h-72 w-72 rounded-lg object-cover {fileId === authState.user.imageId
								? 'ring-primary-500 ring-2'
								: ''}
							"
						/>
					</button>
				{/each}
			</div>
		</div>
	</div>
{/if}

<input
	type="file"
	bind:this={fileInput}
	onchange={handleInputChange}
	style="display: none"
	accept=".jpg,.jpeg,.png,.gif,.bmp,.webp"
/>

<Modal
	open={isUpdatingCategory.open}
	triggerBase="btn preset-tonal"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet content()}
		<header class="flex justify-between">
			<h4 class="h4">Update Profile</h4>
		</header>
		<article>
			<input
				type="text"
				bind:value={isUpdatingCategory.input.username}
				class="input"
			/>
		</article>
		<footer class="flex justify-end gap-4">
			<button
				type="button"
				class="btn preset-tonal"
				onclick={() => (isUpdatingCategory.open = false)}
			>
				Cancel
			</button>
			<button type="button" class="btn preset-filled" onclick={updateProfile}>
				Confirm
			</button>
		</footer>
	{/snippet}
</Modal>
