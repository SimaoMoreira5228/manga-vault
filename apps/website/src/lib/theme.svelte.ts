import { browser } from '$app/environment';

let theme = $state({ theme: 'cerberus', dark: true });

export function toggleDarkMode() {
	theme.dark = !theme.dark;
	saveTheme();
}

export function setTheme(themeName: string) {
	theme.theme = themeName;
	saveTheme();
}

export function getTheme() {
	return theme;
}

export function loadTheme() {
	if (!browser) return;

	try {
		theme = JSON.parse(localStorage.getItem('theme') || '{"theme":"cerberus","dark":true}');
	} catch {
		theme = { theme: 'cerberus', dark: true };
	}
}

function saveTheme() {
	if (!browser) return;
	document.documentElement.setAttribute('data-theme', theme.theme);
	document.documentElement.setAttribute('data-mode', theme.dark ? 'dark' : 'light');
	localStorage.setItem('theme', JSON.stringify(theme));
}
