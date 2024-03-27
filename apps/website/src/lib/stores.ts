import { writable } from 'svelte/store';
import { browser } from '$app/environment';

let themeStoreValue;
if (browser) {
	themeStoreValue = localStorage.theme || 'dark';
} else {
	themeStoreValue = 'dark';
}

export const themeStore = writable(themeStoreValue);

themeStore.subscribe((value) => {
	if (browser) {
		localStorage.theme = value;
	}
});
