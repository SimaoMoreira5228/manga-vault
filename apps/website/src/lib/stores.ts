import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { SortType } from './types';

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

let sortTypeStoreValue: SortType;
if (browser) {
	sortTypeStoreValue = localStorage.sortType || SortType.TITLE;
} else {
	sortTypeStoreValue = SortType.TITLE;
}

export const sortTypeStore = writable(sortTypeStoreValue);

sortTypeStore.subscribe((value) => {
	if (browser) {
		localStorage.sortType = value;
	}
});
