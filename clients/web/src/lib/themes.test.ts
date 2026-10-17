import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { THEMES } from './themes.generated.ts';

function luminance(hex: string): number {
	const channels = [1, 3, 5].map((offset) => {
		const value = parseInt(hex.slice(offset, offset + 2), 16) / 255;
		return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
	});
	return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(first: string, second: string): number {
	const a = luminance(first);
	const b = luminance(second);
	return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

describe('generated themes', () => {
	it('offers ten themes with distinct ids and names', () => {
		assert.equal(THEMES.length, 10);
		assert.equal(new Set(THEMES.map((theme) => theme.id)).size, 10);
		assert.equal(new Set(THEMES.map((theme) => theme.name)).size, 10);
	});

	it('offers both dark and light themes', () => {
		assert.ok(THEMES.some((theme) => theme.mode === 'dark'));
		assert.ok(THEMES.some((theme) => theme.mode === 'light'));
	});

	it('gives every theme a distinct surface colour', () => {
		assert.equal(new Set(THEMES.map((theme) => theme.palette.surface)).size, 10);
	});

	it('keeps body text readable on every surface', () => {
		for (const theme of THEMES) {
			for (const surface of [
				'surface',
				'surfaceLow',
				'surfaceContainer',
				'surfaceHigh',
				'surfaceHighest',
			] as const) {
				assert.ok(
					contrast(theme.palette.onSurface, theme.palette[surface]) >= 4.5,
					`${theme.id}: onSurface on ${surface}`,
				);
			}
		}
	});

	it('keeps secondary text readable on the surfaces it sits on', () => {
		for (const theme of THEMES) {
			for (const surface of ['surface', 'surfaceContainer', 'surfaceHigh'] as const) {
				assert.ok(
					contrast(theme.palette.onSurfaceVariant, theme.palette[surface]) >= 4.5,
					`${theme.id}: onSurfaceVariant on ${surface}`,
				);
			}
		}
	});

	it('keeps accent fills readable and the accent visible on the surface', () => {
		for (const theme of THEMES) {
			assert.ok(contrast(theme.palette.onPrimary, theme.palette.primary) >= 4.5, theme.id);
			assert.ok(
				contrast(theme.palette.onPrimaryContainer, theme.palette.primaryContainer) >= 4.5,
				theme.id,
			);
			assert.ok(contrast(theme.palette.primary, theme.palette.surface) >= 3, theme.id);
			assert.ok(contrast(theme.palette.outline, theme.palette.surface) >= 3, theme.id);
		}
	});

	it('keeps the unread-count pill and tinted chips readable', () => {
		for (const theme of THEMES) {
			assert.ok(
				contrast(theme.palette.onSecondary, theme.palette.secondary) >= 4.5,
				`${theme.id}: onSecondary on secondary`,
			);
			assert.ok(
				contrast(theme.palette.secondary, theme.palette.secondaryTint) >= 4.5,
				`${theme.id}: secondary on secondaryTint`,
			);
		}
	});
});
