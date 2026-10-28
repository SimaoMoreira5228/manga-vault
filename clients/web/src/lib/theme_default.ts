import { THEMES, type ThemeId } from './themes.generated.ts';

export const DEFAULT_THEME: ThemeId = 'monochrome-dark';

export const isThemeId = (value: string | null): value is ThemeId =>
	THEMES.some((theme) => theme.id === value);

export const resolveThemeId = (stored: string | null): ThemeId =>
	isThemeId(stored) ? stored : DEFAULT_THEME;
