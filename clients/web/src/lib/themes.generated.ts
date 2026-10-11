export const THEME_MODES = ['dark', 'light'] as const;

export type ThemeMode = (typeof THEME_MODES)[number];

export type ThemeId =
	| 'classic'
	| 'forest'
	| 'slate'
	| 'midnight'
	| 'crimson'
	| 'ocean'
	| 'terminal'
	| 'sakura'
	| 'parchment'
	| 'monochrome';

export interface VaultTheme {
	readonly id: ThemeId;
	readonly name: string;
	readonly mode: ThemeMode;
	readonly accent: string;
	readonly deep: string;
	readonly palette: Readonly<
		Record<
			| 'surface'
			| 'surfaceDim'
			| 'surfaceBright'
			| 'surfaceLowest'
			| 'surfaceLow'
			| 'surfaceContainer'
			| 'surfaceHigh'
			| 'surfaceHighest'
			| 'onSurface'
			| 'onSurfaceVariant'
			| 'outline'
			| 'outlineVariant'
			| 'primary'
			| 'onPrimary'
			| 'primaryContainer'
			| 'onPrimaryContainer',
			string
		>
	>;
}

export const THEMES: readonly VaultTheme[] = [
	{
		id: 'classic',
		name: 'Classic Vault',
		mode: 'dark',
		accent: '#fab45f',
		deep: '#503a2c',
		palette: {
			surface: '#1b0700',
			surfaceDim: '#1c0700',
			surfaceBright: '#4e301b',
			surfaceLowest: '#0f0100',
			surfaceLow: '#250c00',
			surfaceContainer: '#2d1200',
			surfaceHigh: '#381c06',
			surfaceHighest: '#452813',
			onSurface: '#f1e5de',
			onSurfaceVariant: '#c9b6aa',
			outline: '#99897e',
			outlineVariant: '#503a2c',
			primary: '#fab45f',
			onPrimary: '#351e01',
			primaryContainer: '#955e06',
			onPrimaryContainer: '#fff3e6',
		},
	},
	{
		id: 'forest',
		name: 'Forest Archive',
		mode: 'dark',
		accent: '#9fd780',
		deep: '#344534',
		palette: {
			surface: '#001201',
			surfaceDim: '#011202',
			surfaceBright: '#273f28',
			surfaceLowest: '#000700',
			surfaceLow: '#031905',
			surfaceContainer: '#09200b',
			surfaceHigh: '#132a14',
			surfaceHighest: '#1f3720',
			onSurface: '#e2ebe2',
			onSurfaceVariant: '#b1bfb1',
			outline: '#849084',
			outlineVariant: '#344534',
			primary: '#9fd780',
			onPrimary: '#122b00',
			primaryContainer: '#4c7b2d',
			onPrimaryContainer: '#eafce1',
		},
	},
	{
		id: 'slate',
		name: 'Slate Scholar',
		mode: 'dark',
		accent: '#34dde5',
		deep: '#2c4449',
		palette: {
			surface: '#001115',
			surfaceDim: '#001114',
			surfaceBright: '#173e45',
			surfaceLowest: '#000609',
			surfaceLow: '#00181c',
			surfaceContainer: '#001f25',
			surfaceHigh: '#002a30',
			surfaceHighest: '#0e363d',
			onSurface: '#deebed',
			onSurfaceVariant: '#abbfc2',
			outline: '#7f9093',
			outlineVariant: '#2c4449',
			primary: '#34dde5',
			onPrimary: '#012a2c',
			primaryContainer: '#0a7b80',
			onPrimaryContainer: '#dbfdff',
		},
	},
	{
		id: 'midnight',
		name: 'Midnight Ink',
		mode: 'dark',
		accent: '#bdbcfd',
		deep: '#393e56',
		palette: {
			surface: '#070926',
			surfaceDim: '#080a24',
			surfaceBright: '#2f3557',
			surfaceLowest: '#020215',
			surfaceLow: '#0d102e',
			surfaceContainer: '#131736',
			surfaceHigh: '#1c2141',
			surfaceHighest: '#282d4e',
			onSurface: '#e4e7f4',
			onSurfaceVariant: '#b5b9ce',
			outline: '#878b9d',
			outlineVariant: '#393e56',
			primary: '#bdbcfd',
			onPrimary: '#211c48',
			primaryContainer: '#6660ae',
			onPrimaryContainer: '#f4f4ff',
		},
	},
	{
		id: 'crimson',
		name: 'Crimson Scroll',
		mode: 'dark',
		accent: '#ffa9af',
		deep: '#543734',
		palette: {
			surface: '#210102',
			surfaceDim: '#1f0303',
			surfaceBright: '#532b28',
			surfaceLowest: '#110000',
			surfaceLow: '#290606',
			surfaceContainer: '#310d0c',
			surfaceHigh: '#3c1715',
			surfaceHighest: '#4a2321',
			onSurface: '#f4e4e2',
			onSurfaceVariant: '#cdb4b1',
			outline: '#9c8684',
			outlineVariant: '#543734',
			primary: '#ffa9af',
			onPrimary: '#430e17',
			primaryContainer: '#a54b55',
			onPrimaryContainer: '#fef2f2',
		},
	},
	{
		id: 'ocean',
		name: 'Deep Ocean',
		mode: 'dark',
		accent: '#36dede',
		deep: '#2a4450',
		palette: {
			surface: '#001019',
			surfaceDim: '#00101a',
			surfaceBright: '#133d4f',
			surfaceLowest: '#00060c',
			surfaceLow: '#001722',
			surfaceContainer: '#001e2c',
			surfaceHigh: '#002838',
			surfaceHighest: '#0a3547',
			onSurface: '#ddeaf1',
			onSurfaceVariant: '#a9bec9',
			outline: '#7d8f99',
			outlineVariant: '#2a4450',
			primary: '#36dede',
			onPrimary: '#022a2a',
			primaryContainer: '#0c7b7c',
			onPrimaryContainer: '#d7fefd',
		},
	},
	{
		id: 'terminal',
		name: 'Terminal',
		mode: 'dark',
		accent: '#96d987',
		deep: '#354438',
		palette: {
			surface: '#011205',
			surfaceDim: '#021105',
			surfaceBright: '#293e2d',
			surfaceLowest: '#000701',
			surfaceLow: '#05190a',
			surfaceContainer: '#0b2010',
			surfaceHigh: '#15291a',
			surfaceHighest: '#213625',
			onSurface: '#e3eae4',
			onSurfaceVariant: '#b2beb4',
			outline: '#858f86',
			outlineVariant: '#354438',
			primary: '#96d987',
			onPrimary: '#092d02',
			primaryContainer: '#427c35',
			onPrimaryContainer: '#e8fce3',
		},
	},
	{
		id: 'sakura',
		name: 'Sakura Bloom',
		mode: 'light',
		accent: '#98397e',
		deep: '#caafba',
		palette: {
			surface: '#fff1f7',
			surfaceDim: '#fae9f0',
			surfaceBright: '#e8c1d2',
			surfaceLowest: '#fffafc',
			surfaceLow: '#f9e5ed',
			surfaceContainer: '#f5dde7',
			surfaceHigh: '#eed2de',
			surfaceHighest: '#e7c7d5',
			onSurface: '#2a1921',
			onSurfaceVariant: '#5c4550',
			outline: '#816f76',
			outlineVariant: '#caafba',
			primary: '#98397e',
			onPrimary: '#fffafd',
			primaryContainer: '#feb8e6',
			onPrimaryContainer: '#510840',
		},
	},
	{
		id: 'parchment',
		name: 'Parchment',
		mode: 'light',
		accent: '#925000',
		deep: '#c5b59f',
		palette: {
			surface: '#fcf4e9',
			surfaceDim: '#f7ede0',
			surfaceBright: '#e1caaa',
			surfaceLowest: '#fffbf6',
			surfaceLow: '#f5e9d9',
			surfaceContainer: '#f1e2cf',
			surfaceHigh: '#e9d8c2',
			surfaceHighest: '#e1ceb5',
			onSurface: '#281d0e',
			onSurfaceVariant: '#584b38',
			outline: '#7d7365',
			outlineVariant: '#c5b59f',
			primary: '#925000',
			onPrimary: '#fffbf8',
			primaryContainer: '#fdc496',
			onPrimaryContainer: '#462401',
		},
	},
	{
		id: 'monochrome',
		name: 'Monochrome',
		mode: 'light',
		accent: '#0165af',
		deep: '#b3b8bd',
		palette: {
			surface: '#f3f5f8',
			surfaceDim: '#eceff2',
			surfaceBright: '#c8ced5',
			surfaceLowest: '#fafcfe',
			surfaceLow: '#e8ebef',
			surfaceContainer: '#e1e5e9',
			surfaceHigh: '#d7dbe0',
			surfaceHighest: '#ccd2d7',
			onSurface: '#1d2023',
			onSurfaceVariant: '#4a4d52',
			outline: '#727578',
			outlineVariant: '#b3b8bd',
			primary: '#0165af',
			onPrimary: '#f9fcff',
			primaryContainer: '#afd5fd',
			onPrimaryContainer: '#022f55',
		},
	},
];
