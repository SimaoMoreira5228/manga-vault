import { execFileSync } from 'node:child_process';
import { existsSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, '..', '..');

const SPECS = [
	{ id: 'classic', name: 'Classic Vault', mode: 'dark', hue: 55, accent: 70, chroma: 0.055 },
	{ id: 'forest', name: 'Forest Archive', mode: 'dark', hue: 145, accent: 135, chroma: 0.05 },
	{ id: 'slate', name: 'Slate Scholar', mode: 'dark', hue: 210, accent: 200, chroma: 0.045 },
	{ id: 'midnight', name: 'Midnight Ink', mode: 'dark', hue: 275, accent: 285, chroma: 0.06 },
	{ id: 'crimson', name: 'Crimson Scroll', mode: 'dark', hue: 25, accent: 15, chroma: 0.06 },
	{ id: 'ocean', name: 'Deep Ocean', mode: 'dark', hue: 230, accent: 195, chroma: 0.055 },
	{ id: 'terminal', name: 'Terminal', mode: 'dark', hue: 150, accent: 140, chroma: 0.04 },
	{ id: 'sakura', name: 'Sakura Bloom', mode: 'light', hue: 350, accent: 340, chroma: 0.05 },
	{ id: 'parchment', name: 'Parchment', mode: 'light', hue: 75, accent: 60, chroma: 0.05 },
	{ id: 'monochrome', name: 'Monochrome', mode: 'light', hue: 250, accent: 250, chroma: 0.012 },
];

function run(command, args, options = {}) {
	try {
		execFileSync(command, args, { stdio: 'ignore', ...options });
		return 0;
	} catch (error) {
		if (error.status === 1) return 1;
		throw error;
	}
}

const clamp = (value, low, high) => Math.min(high, Math.max(low, value));

function oklchToRgb(L, C, H) {
	const hRad = (H * Math.PI) / 180;
	const a = C * Math.cos(hRad);
	const b = C * Math.sin(hRad);
	const lRoot = L + 0.3963377774 * a + 0.2158037573 * b;
	const mRoot = L - 0.1055613458 * a - 0.0638541728 * b;
	const sRoot = L - 0.0894841775 * a - 1.291485548 * b;
	const l = lRoot ** 3;
	const m = mRoot ** 3;
	const s = sRoot ** 3;
	const linear = [
		4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
	];
	return linear.map((channel) => {
		const encoded =
			channel <= 0.0031308 ? 12.92 * channel : 1.055 * channel ** (1 / 2.4) - 0.055;
		return clamp(Math.round(encoded * 255), 0, 255);
	});
}

const inGamut = (L, C, H) => {
	const hRad = (H * Math.PI) / 180;
	const a = C * Math.cos(hRad);
	const b = C * Math.sin(hRad);
	const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
	const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
	const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
	const linear = [
		4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
	];
	return linear.every((channel) => channel >= -0.0005 && channel <= 1.0005);
};

function oklch(L, C, H) {
	let chroma = C;
	let lightness = L;
	for (let step = 0; step < 24 && !inGamut(lightness, chroma, H); step += 1) {
		chroma *= 0.94;
	}
	const [r, g, b] = oklchToRgb(lightness, chroma, H);
	return {
		hex: `#${[r, g, b].map((value) => value.toString(16).padStart(2, '0')).join('')}`,
		rgb: [r, g, b],
	};
}

function luminance(rgb) {
	const [r, g, b] = rgb.map((channel) => {
		const c = channel / 255;
		return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
	});
	return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a, b) {
	const first = luminance(a.rgb);
	const second = luminance(b.rgb);
	const [light, dark] = first > second ? [first, second] : [second, first];
	return (light + 0.05) / (dark + 0.05);
}

function paletteOf(spec) {
	const dark = spec.mode === 'dark';
	const tone = (lightness, chromaScale = 1) =>
		oklch(lightness, spec.chroma * chromaScale, spec.hue);
	const accent = (lightness, chroma) => oklch(lightness, chroma, spec.accent);

	if (dark) {
		return {
			surface: tone(0.16),
			surfaceDim: tone(0.16, 0.9),
			surfaceBright: tone(0.34),
			surfaceLowest: tone(0.11, 0.8),
			surfaceLow: tone(0.19),
			surfaceContainer: tone(0.22),
			surfaceHigh: tone(0.26),
			surfaceHighest: tone(0.31),
			onSurface: tone(0.93, 0.3),
			onSurfaceVariant: tone(0.79, 0.5),
			outline: tone(0.64, 0.45),
			outlineVariant: tone(0.37, 0.7),
			primary: accent(0.82, 0.13),
			onPrimary: accent(0.26, 0.08),
			primaryContainer: accent(0.53, 0.12),
			onPrimaryContainer: accent(0.97, 0.04),
		};
	}
	return {
		surface: tone(0.97, 0.35),
		surfaceDim: tone(0.95, 0.4),
		surfaceBright: tone(0.85),
		surfaceLowest: tone(0.99, 0.3),
		surfaceLow: tone(0.94, 0.5),
		surfaceContainer: tone(0.92, 0.6),
		surfaceHigh: tone(0.89, 0.7),
		surfaceHighest: tone(0.86, 0.8),
		onSurface: tone(0.24, 0.6),
		onSurfaceVariant: tone(0.42, 0.7),
		outline: tone(0.56, 0.5),
		outlineVariant: tone(0.78, 0.7),
		primary: accent(0.5, 0.15),
		onPrimary: accent(0.99, 0.02),
		primaryContainer: accent(0.86, 0.1),
		onPrimaryContainer: accent(0.3, 0.12),
	};
}

const REQUIRED_PAIRS = [
	['onSurface', 'surface', 4.5],
	['onSurface', 'surfaceLow', 4.5],
	['onSurface', 'surfaceContainer', 4.5],
	['onSurface', 'surfaceHigh', 4.5],
	['onSurface', 'surfaceHighest', 4.5],
	['onSurfaceVariant', 'surface', 4.5],
	['onSurfaceVariant', 'surfaceContainer', 4.5],
	['onSurfaceVariant', 'surfaceHigh', 4.5],
	['onPrimary', 'primary', 4.5],
	['onPrimaryContainer', 'primaryContainer', 4.5],
	['primary', 'surface', 3],
	['outline', 'surface', 3],
	['outlineVariant', 'surface', 1.4],
];

const failures = [];
const palettes = SPECS.map((spec) => {
	const palette = paletteOf(spec);
	for (const [fg, bg, minimum] of REQUIRED_PAIRS) {
		const ratio = contrast(palette[fg], palette[bg]);
		if (ratio < minimum) {
			failures.push(
				`${spec.id}: ${fg} on ${bg} is ${ratio.toFixed(2)}:1, needs ${minimum}:1`,
			);
		}
	}
	return { spec, palette };
});

if (failures.length > 0) {
	console.error(failures.join('\n'));
	process.exit(1);
}

const TOKENS = [
	'surface',
	'surfaceDim',
	'surfaceBright',
	'surfaceLowest',
	'surfaceLow',
	'surfaceContainer',
	'surfaceHigh',
	'surfaceHighest',
	'onSurface',
	'onSurfaceVariant',
	'outline',
	'outlineVariant',
	'primary',
	'onPrimary',
	'primaryContainer',
	'onPrimaryContainer',
];

const ts = `export const THEME_MODES = ['dark', 'light'] as const;

export type ThemeMode = (typeof THEME_MODES)[number];

export type ThemeId = ${SPECS.map((spec) => `'${spec.id}'`).join(' | ')};

export interface VaultTheme {
	readonly id: ThemeId;
	readonly name: string;
	readonly mode: ThemeMode;
	readonly accent: string;
	readonly deep: string;
	readonly palette: Readonly<Record<${TOKENS.map((token) => `'${token}'`).join('|')}, string>>;
}

export const THEMES: readonly VaultTheme[] = ${JSON.stringify(
	palettes.map(({ spec, palette }) => ({
		id: spec.id,
		name: spec.name,
		mode: spec.mode,
		accent: palette.primary.hex,
		deep: palette.outlineVariant.hex,
		palette: Object.fromEntries(TOKENS.map((token) => [token, palette[token].hex])),
	})),
	null,
	'\t',
)};

`;

const cssVariable = (token) => `--color-${token.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`;

const tokensBlock = (palette) =>
	TOKENS.map((token) => `\t${cssVariable(token)}: ${palette[token].hex};`).join('\n');

const css = `/* Generated by tools/themes/build.mjs. Do not edit. */

@theme {
${tokensBlock(palettes[0].palette)}
}

${palettes
	.map(
		({ spec, palette }) => `[data-theme='${spec.id}'] {
	color-scheme: ${spec.mode};
${tokensBlock(palette)}
}
`,
	)
	.join('\n')}`;

const dart = `// Generated by tools/themes/build.mjs. Do not edit.

import 'package:flutter/material.dart';

import 'theme.dart';

const _palettes = <String, VaultPalette>{
${palettes
	.map(
		({ spec, palette }) => `	'${spec.id}': VaultPalette(
${TOKENS.map((token) => `		${token}: Color(0xFF${palette[token].hex.slice(1).toUpperCase()}),`).join('\n')}
	),`,
	)
	.join('\n')}
};

const vaultPaletteEntries = <(String, String)>[
${palettes.map(({ spec }) => `	('${spec.id}', '${spec.name.replace(/'/g, "\\'")}'),`).join('\n')}
];

VaultPalette paletteForThemeId(String id) => _palettes[id] ?? _palettes['classic']!;

const vaultThemeModes = <String, VaultThemeMode>{
${palettes.map(({ spec }) => `	'${spec.id}': VaultThemeMode.${spec.mode},`).join('\n')}
};
`;

const webOutputs = [
	join(repo, 'clients/web/src/lib/themes.generated.ts'),
	join(repo, 'clients/web/src/routes/themes.generated.css'),
];
const dartOutput = join(repo, 'clients/flutter/lib/theme_palette_data.g.dart');

writeFileSync(webOutputs[0], ts);
writeFileSync(webOutputs[1], css);
writeFileSync(dartOutput, dart);

const web = join(repo, 'clients/web');
const biome = join(web, 'node_modules/.bin/biome');
if (!existsSync(biome)) {
	console.error('biome is missing, run pnpm install in clients/web first');
	process.exit(1);
}
const relative = (path) => path.slice(web.length + 1);
const webArgs = webOutputs.map(relative);
run(biome, ['format', '--write', ...webArgs], { cwd: web });
if (run(biome, ['format', ...webArgs], { cwd: web }) !== 0) {
	console.error('generated web output is not biome clean');
	process.exit(1);
}
run('dart', ['format', dartOutput]);
if (run('dart', ['format', '--output=none', '--set-exit-if-changed', dartOutput]) !== 0) {
	console.error('generated dart output is not dart-format clean');
	process.exit(1);
}

const tightest = new Map();
for (const { spec, palette } of palettes) {
	for (const [fg, bg, minimum] of REQUIRED_PAIRS) {
		const ratio = contrast(palette[fg], palette[bg]);
		const previous = tightest.get(`${fg} on ${bg}`);
		if (previous === undefined || ratio < previous.ratio) {
			tightest.set(`${fg} on ${bg}`, { ratio, minimum, theme: spec.id });
		}
	}
}
const summary = [...tightest.entries()]
	.map(([pair, { ratio, minimum }]) => `${pair} ${ratio.toFixed(2)}:1 (needs ${minimum})`)
	.sort();
console.log(`${palettes.length} themes written. Tightest of each required pair:`);
console.log(summary.map((line) => `  ${line}`).join('\n'));