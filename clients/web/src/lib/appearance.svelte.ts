import { DEFAULT_THEME, resolveThemeId } from './theme_default.ts';
import { THEMES, type ThemeId } from './themes.generated.ts';

const THEME_KEY = 'mv-theme';
const PURE_BLACK_KEY = 'mv-pure-black';

class AppearanceState {
	theme = $state<ThemeId>(DEFAULT_THEME);
	pureBlack = $state(false);

	constructor() {
		const stored = localStorage.getItem(THEME_KEY);
		this.theme = resolveThemeId(stored);
		this.pureBlack = localStorage.getItem(PURE_BLACK_KEY) === '1';
		document.documentElement.dataset.theme = this.theme;
		if (this.pureBlack) {
			document.documentElement.dataset.pureBlack = '';
		}
	}

	setTheme(theme: ThemeId) {
		this.theme = theme;
		localStorage.setItem(THEME_KEY, theme);
		document.documentElement.dataset.theme = theme;
		this.setPureBlack(this.pureBlack);
	}

	setPureBlack(value: boolean) {
		this.pureBlack = value;
		if (value && THEMES.some((theme) => theme.id === this.theme && theme.mode === 'light')) {
			this.pureBlack = false;
			localStorage.removeItem(PURE_BLACK_KEY);
			delete document.documentElement.dataset.pureBlack;
			return;
		}
		if (this.pureBlack) {
			localStorage.setItem(PURE_BLACK_KEY, '1');
			document.documentElement.dataset.pureBlack = '';
		} else {
			localStorage.removeItem(PURE_BLACK_KEY);
			delete document.documentElement.dataset.pureBlack;
		}
	}
}

export const appearance = new AppearanceState();

export type { ThemeId };
export { THEMES };
