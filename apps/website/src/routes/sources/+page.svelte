<script lang="ts">
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { Tabs } from "@skeletonlabs/skeleton-svelte";
import type { PageData } from "./$types";

let props = $props();
let data: PageData = props.data;

let scrapers = $derived.by(() => (data.scrapers || []).sort((a, b) => a.name.localeCompare(b.name)));
let group = $state("manga");
</script>

<div class="flex h-full w-full items-center justify-center">
	<div class="flex h-full w-full flex-col items-start justify-start gap-4 overflow-auto p-4">
		<Tabs value={group} onValueChange={(e) => (group = e.value)}>
			{#snippet list()}
				<Tabs.Control value="manga">Mangas</Tabs.Control>
				<Tabs.Control value="novel">Novels</Tabs.Control>
			{/snippet}

			{#snippet content()}
				<Tabs.Panel value="manga">
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
				</Tabs.Panel>

				<Tabs.Panel value="novel">
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
				</Tabs.Panel>
			{/snippet}
		</Tabs>
	</div>
</div>
