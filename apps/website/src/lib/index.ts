import { writable, type Writable } from 'svelte/store';
import { colorScheme } from '@svelteuidev/core';

function safeParseJSON<T>(json: string | null, fallback: T): T {
	if (json === null) return fallback;
	try {
		return JSON.parse(json);
	} catch {
		return fallback;
	}
}

export class Store<T> {
	private value: T;
	private readonly useLocalStorage: boolean = false;
	private readonly name: string;
	store: Writable<T>;

	constructor(
		value: T,
		useLocalStorage: boolean = false,
		name: string = '',
		callback?: (value: T) => void
	) {
		this.value = value;
		this.useLocalStorage = useLocalStorage;
		this.name = name;

		this.store = writable(this.value);

		if (this.useLocalStorage) {
			localStorage.setItem(this.name, JSON.stringify(value));
		}

		if (callback) {
			this.store.subscribe(callback);
		}
	}

	set(value: T) {
		this.value = value;
		this.store.set(value);

		if (this.useLocalStorage) {
			localStorage.setItem(this.name, JSON.stringify(value));
		}

		return this.value;
	}

	get() {
		return this.value;
	}

	setCallback(callback: (value: T) => void) {
		this.store.subscribe(callback);
	}
}

export const themeStore = new Store<'dark' | 'light'>(
	safeParseJSON(localStorage.getItem('theme'), 'dark'),
	true,
	'theme',
	(value) => {
		colorScheme.set(value);
	}
);
