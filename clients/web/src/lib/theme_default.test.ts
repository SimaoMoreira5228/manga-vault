import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { DEFAULT_THEME, isThemeId, resolveThemeId } from './theme_default.ts';
import { THEMES } from './themes.generated.ts';

describe('theme default', () => {
	it('defaults to a dark theme', () => {
		assert.equal(DEFAULT_THEME, 'monochrome-dark');
		assert.equal(THEMES.find((theme) => theme.id === DEFAULT_THEME)?.mode, 'dark');
	});

	it('falls back to the default when nothing is stored', () => {
		assert.equal(resolveThemeId(null), DEFAULT_THEME);
	});

	it('falls back to the default when the stored id is not a theme', () => {
		assert.equal(resolveThemeId('sunset'), DEFAULT_THEME);
		assert.equal(resolveThemeId(''), DEFAULT_THEME);
	});

	it('keeps a stored choice', () => {
		for (const theme of THEMES) {
			assert.equal(resolveThemeId(theme.id), theme.id);
		}
	});

	it('recognises exactly the generated ids', () => {
		assert.equal(isThemeId('classic'), true);
		assert.equal(isThemeId('sunset'), false);
		assert.equal(isThemeId(null), false);
	});
});
