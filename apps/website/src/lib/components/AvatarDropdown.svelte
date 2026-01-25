<script lang="ts">
import { resolve } from "$app/paths";
import type { AuthState } from "$lib/auth.svelte";
import { CircleUserRound, UserRound } from "@lucide/svelte";
import { Avatar, DropdownMenu } from "bits-ui";

let {
	authState,
	getImage,
	logout,
	size,
}: {
	authState: AuthState;
	getImage: (id: number) => string;
	logout: () => void;
	size?: "lg" | "md";
} = $props();

const avatarSizes = {
	lg: { root: "h-20 w-20", img: "h-20 w-20", fallback: "h-20 w-20", fallbackIconSize: 96 },
	md: { root: "h-16 w-16", img: "h-16 w-16", fallback: "h-12 w-12", fallbackIconSize: 64 },
};

$effect(() => {
	size = size ?? "lg";
});
</script>

{#if authState?.status === "authenticated"}
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#if authState.user?.imageId}
				<Avatar.Root delayMs={200} class={avatarSizes[size ?? "lg"].root + " rounded-full"}>
					<div class="flex h-full w-full items-center justify-center overflow-hidden rounded-full border-transparent">
						<Avatar.Image
							src={getImage(authState.user.imageId ?? 0)}
							alt=""
							class={avatarSizes[size ?? "lg"].img + " rounded-full object-cover"}
						/>
						<Avatar.Fallback
							class={"flex " + avatarSizes[size ?? "lg"].fallback + " items-center justify-center rounded-full"}
						>
							<UserRound size={avatarSizes[size ?? "lg"].fallbackIconSize} />
						</Avatar.Fallback>
					</div>
				</Avatar.Root>
			{:else}
				<Avatar.Root delayMs={200} class={avatarSizes[size ?? "lg"].root + " rounded-full border"}>
					<div class="flex h-full w-full items-center justify-center overflow-hidden rounded-full border-2 border-transparent">
						<Avatar.Fallback
							class={"flex " + avatarSizes[size ?? "lg"].fallback + " items-center justify-center rounded-full"}
						>
							<UserRound size={avatarSizes[size ?? "lg"].fallbackIconSize} />
						</Avatar.Fallback>
					</div>
				</Avatar.Root>
			{/if}
		</DropdownMenu.Trigger>

		<DropdownMenu.Portal>
			<DropdownMenu.Content
				class="bg-surface-50-950 border-surface-200-800 z-50 w-56 border p-1 shadow-lg"
				sideOffset={5}
				side="left"
			>
				<DropdownMenu.Item
					class="hover:bg-surface-300-700 focus:bg-surface-200-800 outline-hidden flex cursor-pointer items-center rounded-md px-2 py-2 text-sm transition-colors"
				>
					<a
						href={resolve("/profile")}
						class="flex w-full flex-row items-center justify-start"
					>
						<CircleUserRound class="mr-2 h-4 w-4" />
						<span>Profile</span>
					</a>
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="bg-surface-200-800 my-1 h-px" />
				<DropdownMenu.Item
					class="hover:bg-surface-300-700 focus:bg-surface-200-800 outline-hidden flex cursor-pointer items-center rounded-md px-2 py-2 text-sm transition-colors"
				>
					<button onclick={logout}>Log Out</button>
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>
{/if}
