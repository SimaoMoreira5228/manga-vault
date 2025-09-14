<script lang="ts">
import { getTheme, setTheme, toggleDarkMode } from "$lib/theme.svelte";
import { Moon, Sun } from "@lucide/svelte";
let { expanded = true }: { expanded: boolean | undefined } = $props();
let theme = $derived(getTheme());

const THEME_LIST = [
	"catppuccin",
	"cerberus",
	"concord",
	"crimson",
	"fennec",
	"hamlindigo",
	"legacy",
	"mint",
	"modern",
	"mona",
	"nosh",
	"nouveau",
	"pine",
	"reign",
	"rocket",
	"rose",
	"sahara",
	"seafoam",
	"terminus",
	"vintage",
	"vox",
	"wintry",
];

function onThemeChange(e: Event) {
	const v = (e.target as HTMLSelectElement).value;
	setTheme(v);
}
function toggleDark() {
	toggleDarkMode();
}
</script>

{#if expanded}
	<div class="flex w-full flex-row items-end justify-center gap-2">
		<label class="label">
			<span class="label-text">Theme</span>
			<select class="select" onchange={onThemeChange} bind:value={theme.theme}>
				{#each THEME_LIST as t (t)}
					<option value={t}>{t[0].toUpperCase() + t.slice(1)}</option>
				{/each}
			</select>
		</label>

		{#if theme.dark}
			<button type="button" class="btn-icon preset-filled" onclick={toggleDark}>
				<Sun />
			</button>
		{:else}
			<button type="button" class="btn-icon preset-filled" onclick={toggleDark}>
				<Moon />
			</button>
		{/if}
	</div>
{/if}
