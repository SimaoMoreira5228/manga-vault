export const THEMES = [
	{ id: 'classic', name: 'Classic Vault', accent: '#ffb77d', deep: '#554336' },
	{ id: 'forest', name: 'Forest Archive', accent: '#a8d5a2', deep: '#2c3f30' },
	{ id: 'slate', name: 'Slate Scholar', accent: '#80c8d4', deep: '#23353b' },
] as const;

export type ThemeId = (typeof THEMES)[number]['id'];

const THEME_KEY = 'mv-theme';
const PURE_BLACK_KEY = 'mv-pure-black';

class AppearanceState {
	theme = $state<ThemeId>('classic');
	pureBlack = $state(false);

	constructor() {
		const stored = localStorage.getItem(THEME_KEY) as ThemeId | null;
		this.theme = THEMES.some((theme) => theme.id === stored) ? (stored as ThemeId) : 'classic';
		this.pureBlack = localStorage.getItem(PURE_BLACK_KEY) === '1';
	}

	setTheme(theme: ThemeId) {
		this.theme = theme;
		localStorage.setItem(THEME_KEY, theme);
		document.documentElement.dataset.theme = theme;
	}

	setPureBlack(value: boolean) {
		this.pureBlack = value;
		if (value) {
			localStorage.setItem(PURE_BLACK_KEY, '1');
			document.documentElement.dataset.pureBlack = '';
		} else {
			localStorage.removeItem(PURE_BLACK_KEY);
			delete document.documentElement.dataset.pureBlack;
		}
	}
}

export const appearance = new AppearanceState();
