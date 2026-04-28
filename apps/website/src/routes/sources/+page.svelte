<script lang="ts">
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { Tabs } from "@skeletonlabs/skeleton-svelte";
import type { PageData } from "./$types";

let { data }: { data: PageData } = $props();

let scrapers = $derived.by(() => (data.scrapers || []).sort((a, b) => a.name.localeCompare(b.name)));
let group = $state("manga");
</script>

<div class="flex h-full w-full items-center justify-center">
	<div class="flex h-full w-full flex-col items-start justify-start gap-4 overflow-auto p-4">
		<Tabs value={group} onValueChange={(e) => (group = e.value)} class="flex flex-col w-full">
			<Tabs.List class="flex gap-2">
				<Tabs.Trigger
					value="manga"
					class={`btn flex flex-1 items-center justify-between border-b-2 ${
						group === "manga"
							? "border-primary-500"
							: "border-transparent"
					}`}
				>Mangas</Tabs.Trigger>
				<Tabs.Trigger
					value="novel"
					class={`btn flex flex-1 items-center justify-between border-b-2 ${
						group === "novel"
							? "border-primary-500"
							: "border-transparent"
					}`}
				>Novels</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="manga">
				<div class="flex flex-col w-full h-full gap-2 mt-6">
					{#each scrapers.filter(s => s.type === "MANGA") as scraper (scraper.id)}
						<a
							class="card preset-filled-surface-100-900 flex w-full items-center justify-between space-x-4 p-4 text-center"
							href={resolve(`/sources/${scraper.id}/trending`)}
						>
							<div class="flex items-center space-x-4">
								<img
									src={scraper.imageUrl}
									alt={scraper.name}
									class="h-12 max-w-12 rounded"
								/>
								<h5 class="h5">{scraper.name}</h5>
							</div>
							<button
								type="button"
								class="btn preset-tonal-primary"
								onclick={(e) => {
									e.preventDefault();
									goto(resolve(`/sources/${scraper.id}/latest`));
								}}
							>
								Latest
							</button>
						</a>
					{/each}
				</div>
			</Tabs.Content>

			<Tabs.Content value="novel">
				<div class="flex flex-col w-full h-full gap-2 mt-6">
					{#each scrapers.filter(s => s.type === "NOVEL") as scraper (scraper.id)}
						<a
							class="card preset-filled-surface-100-900 flex w-full items-center justify-between space-x-4 p-4 text-center"
							href={resolve(`/sources/${scraper.id}/trending`)}
						>
							<div class="flex items-center space-x-4">
								<img
									src={scraper.imageUrl}
									alt={scraper.name}
									class="h-12 max-w-12 rounded"
								/>
								<h5 class="h5">{scraper.name}</h5>
							</div>
							<button
								type="button"
								class="btn preset-tonal-primary"
								onclick={(e) => {
									e.preventDefault();
									goto(resolve(`/sources/${scraper.id}/latest`));
								}}
							>
								Latest
							</button>
						</a>
					{/each}
				</div>
			</Tabs.Content>
		</Tabs>
	</div>
</div>
