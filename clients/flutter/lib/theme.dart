import 'package:flutter/material.dart';

import 'service/app_prefs.dart';

const _geist = 'Geist';
const _playfair = 'Playfair Display';

const _secondary = Color(0xFF44E2CD);
const _onSecondary = Color(0xFF003731);
const _secondaryContainer = Color(0xFF03C6B2);
const _onSecondaryContainer = Color(0xFF004D44);
const _error = Color(0xFFFFB4AB);
const _onError = Color(0xFF690005);
const _errorContainer = Color(0xFF93000A);
const _onErrorContainer = Color(0xFFFFDAD6);
const _black = Color(0xFF000000);

class VaultPalette {
  const VaultPalette({
    required this.surface,
    required this.surfaceDim,
    required this.surfaceBright,
    required this.surfaceLowest,
    required this.surfaceLow,
    required this.surfaceContainer,
    required this.surfaceHigh,
    required this.surfaceHighest,
    required this.onSurface,
    required this.onSurfaceVariant,
    required this.outline,
    required this.outlineVariant,
    required this.primary,
    required this.onPrimary,
    required this.primaryContainer,
    required this.onPrimaryContainer,
  });

  final Color surface;
  final Color surfaceDim;
  final Color surfaceBright;
  final Color surfaceLowest;
  final Color surfaceLow;
  final Color surfaceContainer;
  final Color surfaceHigh;
  final Color surfaceHighest;
  final Color onSurface;
  final Color onSurfaceVariant;
  final Color outline;
  final Color outlineVariant;
  final Color primary;
  final Color onPrimary;
  final Color primaryContainer;
  final Color onPrimaryContainer;
}

class VaultTheme {
  const VaultTheme({
    required this.id,
    required this.name,
    required this.palette,
  });

  final String id;
  final String name;
  final VaultPalette palette;
}

const classicTheme = VaultTheme(
  id: 'classic',
  name: 'Classic Vault',
  palette: VaultPalette(
    surface: Color(0xFF1A120B),
    surfaceDim: Color(0xFF1A120B),
    surfaceBright: Color(0xFF42372F),
    surfaceLowest: Color(0xFF150C06),
    surfaceLow: Color(0xFF231A13),
    surfaceContainer: Color(0xFF271E16),
    surfaceHigh: Color(0xFF322820),
    surfaceHighest: Color(0xFF3E332B),
    onSurface: Color(0xFFF2DFD3),
    onSurfaceVariant: Color(0xFFDBC2B0),
    outline: Color(0xFFA38C7C),
    outlineVariant: Color(0xFF554336),
    primary: Color(0xFFFFB77D),
    onPrimary: Color(0xFF4D2600),
    primaryContainer: Color(0xFFD97707),
    onPrimaryContainer: Color(0xFF432100),
  ),
);

const forestTheme = VaultTheme(
  id: 'forest',
  name: 'Forest Archive',
  palette: VaultPalette(
    surface: Color(0xFF111812),
    surfaceDim: Color(0xFF111812),
    surfaceBright: Color(0xFF33463A),
    surfaceLowest: Color(0xFF0C110D),
    surfaceLow: Color(0xFF141C16),
    surfaceContainer: Color(0xFF18231A),
    surfaceHigh: Color(0xFF1E2C20),
    surfaceHighest: Color(0xFF253627),
    onSurface: Color(0xFFDCEBDD),
    onSurfaceVariant: Color(0xFFB0C4AF),
    outline: Color(0xFF8FA48C),
    outlineVariant: Color(0xFF2C3F30),
    primary: Color(0xFFA8D5A2),
    onPrimary: Color(0xFF0D2A13),
    primaryContainer: Color(0xFF63A86B),
    onPrimaryContainer: Color(0xFF081508),
  ),
);

const slateTheme = VaultTheme(
  id: 'slate',
  name: 'Slate Scholar',
  palette: VaultPalette(
    surface: Color(0xFF0D1518),
    surfaceDim: Color(0xFF0D1518),
    surfaceBright: Color(0xFF2B4048),
    surfaceLowest: Color(0xFF080F12),
    surfaceLow: Color(0xFF10191D),
    surfaceContainer: Color(0xFF142024),
    surfaceHigh: Color(0xFF19282D),
    surfaceHighest: Color(0xFF1F3238),
    onSurface: Color(0xFFD5E4E8),
    onSurfaceVariant: Color(0xFFADC0C7),
    outline: Color(0xFF8AA4AA),
    outlineVariant: Color(0xFF23353B),
    primary: Color(0xFF80C8D4),
    onPrimary: Color(0xFF00363D),
    primaryContainer: Color(0xFF56A9BA),
    onPrimaryContainer: Color(0xFF03242A),
  ),
);

const vaultThemes = [classicTheme, forestTheme, slateTheme];

VaultTheme vaultThemeById(String id) => vaultThemes.firstWhere(
  (theme) => theme.id == id,
  orElse: () => classicTheme,
);

class Appearance {
  const Appearance({required this.themeId, required this.pureBlack});

  final String themeId;
  final bool pureBlack;

  ThemeData build() =>
      buildVaultTheme(vaultThemeById(themeId), pureBlack: pureBlack);
}

final appearance = ValueNotifier<Appearance>(
  const Appearance(themeId: 'classic', pureBlack: false),
);

Future<void> applyAppearance({String? themeId, bool? pureBlack}) async {
  final prefs = await AppPrefs.instance();
  final current = appearance.value;
  final next = Appearance(
    themeId: themeId ?? current.themeId,
    pureBlack: pureBlack ?? current.pureBlack,
  );
  appearance.value = next;
  await prefs.setThemeId(next.themeId);
  await prefs.setPureBlack(next.pureBlack);
}

Future<void> restoreAppearance() async {
  final prefs = await AppPrefs.instance();
  appearance.value = Appearance(
    themeId: prefs.themeId,
    pureBlack: prefs.pureBlack,
  );
}

ThemeData buildVaultTheme(VaultTheme theme, {bool pureBlack = false}) {
  final palette = theme.palette;
  final scheme = ColorScheme.dark(
    surface: pureBlack ? _black : palette.surface,
    surfaceDim: pureBlack ? _black : palette.surfaceDim,
    surfaceBright: palette.surfaceBright,
    surfaceContainerLowest: pureBlack ? _black : palette.surfaceLowest,
    surfaceContainerLow: palette.surfaceLow,
    surfaceContainer: palette.surfaceContainer,
    surfaceContainerHigh: palette.surfaceHigh,
    surfaceContainerHighest: palette.surfaceHighest,
    onSurface: palette.onSurface,
    onSurfaceVariant: palette.onSurfaceVariant,
    primary: palette.primary,
    onPrimary: palette.onPrimary,
    primaryContainer: palette.primaryContainer,
    onPrimaryContainer: palette.onPrimaryContainer,
    secondary: _secondary,
    onSecondary: _onSecondary,
    secondaryContainer: _secondaryContainer,
    onSecondaryContainer: _onSecondaryContainer,
    error: _error,
    onError: _onError,
    errorContainer: _errorContainer,
    onErrorContainer: _onErrorContainer,
    outline: palette.outline,
    outlineVariant: palette.outlineVariant,
    inverseSurface: palette.onSurface,
    onInverseSurface: Color.alphaBlend(
      Colors.black.withValues(alpha: 0.35),
      palette.outlineVariant,
    ),
    surfaceTint: palette.primary,
  );

  final geist = TextStyle(fontFamily: _geist, color: palette.onSurface);
  final playfair = TextStyle(fontFamily: _playfair, color: palette.onSurface);

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: scheme.surface,
    textTheme: TextTheme(
      displayLarge: playfair.copyWith(
        fontSize: 48,
        fontWeight: FontWeight.w700,
        height: 56 / 48,
        letterSpacing: -0.02 * 48,
      ),
      displayMedium: playfair.copyWith(
        fontSize: 32,
        fontWeight: FontWeight.w700,
        height: 40 / 32,
      ),
      displaySmall: playfair.copyWith(
        fontSize: 28,
        fontWeight: FontWeight.w700,
        height: 34 / 28,
      ),
      headlineMedium: geist.copyWith(
        fontSize: 20,
        fontWeight: FontWeight.w600,
        height: 28 / 20,
      ),
      titleLarge: geist.copyWith(
        fontSize: 20,
        fontWeight: FontWeight.w600,
        height: 28 / 20,
      ),
      titleMedium: geist.copyWith(
        fontSize: 16,
        fontWeight: FontWeight.w600,
        height: 24 / 16,
      ),
      bodyLarge: geist.copyWith(
        fontSize: 16,
        fontWeight: FontWeight.w400,
        height: 26 / 16,
      ),
      bodyMedium: geist.copyWith(
        fontSize: 14,
        fontWeight: FontWeight.w400,
        height: 22 / 14,
      ),
      labelSmall: geist.copyWith(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        height: 16 / 12,
        letterSpacing: 0.05 * 12,
      ),
    ),
    appBarTheme: AppBarTheme(
      backgroundColor: scheme.surface,
      foregroundColor: palette.onSurface,
      elevation: 0,
      titleTextStyle: playfair.copyWith(
        fontSize: 22,
        fontWeight: FontWeight.w700,
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: palette.surfaceLow,
      indicatorColor: Colors.transparent,
      iconTheme: WidgetStateProperty.resolveWith(
        (states) => IconThemeData(
          color: states.contains(WidgetState.selected)
              ? palette.primary
              : palette.onSurfaceVariant,
        ),
      ),
      labelTextStyle: WidgetStateProperty.resolveWith(
        (states) => TextStyle(
          fontFamily: _geist,
          fontSize: 12,
          fontWeight: FontWeight.w500,
          color: states.contains(WidgetState.selected)
              ? palette.primary
              : palette.onSurfaceVariant,
        ),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: palette.surfaceLow,
      hintStyle: geist.copyWith(color: palette.outline),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: palette.outlineVariant),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: palette.outlineVariant),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: palette.primary),
      ),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: palette.surfaceHigh,
      labelStyle: geist.copyWith(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        color: palette.onSurfaceVariant,
      ),
      side: BorderSide(color: palette.outlineVariant),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
    listTileTheme: ListTileThemeData(iconColor: palette.onSurfaceVariant),
    cardTheme: CardThemeData(
      color: palette.surfaceLow,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: palette.outlineVariant),
      ),
    ),
  );
}
