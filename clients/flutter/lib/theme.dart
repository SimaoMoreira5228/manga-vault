import 'package:flutter/material.dart';

const surface = Color(0xFF1A120B);
const surfaceContainerLowest = Color(0xFF150C06);
const surfaceContainerLow = Color(0xFF231A13);
const surfaceContainer = Color(0xFF271E16);
const surfaceContainerHigh = Color(0xFF322820);
const surfaceContainerHighest = Color(0xFF3E332B);
const onSurface = Color(0xFFF2DFD3);
const onSurfaceVariant = Color(0xFFDBC2B0);
const outline = Color(0xFFA38C7C);
const outlineVariant = Color(0xFF554336);
const primary = Color(0xFFFFB77D);
const onPrimary = Color(0xFF4D2600);
const primaryContainer = Color(0xFFD97707);
const onPrimaryContainer = Color(0xFF432100);
const secondary = Color(0xFF44E2CD);
const onSecondary = Color(0xFF003731);
const secondaryContainer = Color(0xFF03C6B2);
const onSecondaryContainer = Color(0xFF004D44);
const error = Color(0xFFFFB4AB);
const onError = Color(0xFF690005);
const errorContainer = Color(0xFF93000A);
const onErrorContainer = Color(0xFFFFDAD6);

const _geist = 'Geist';
const _playfair = 'Playfair Display';

ThemeData buildVaultTheme() {
  const scheme = ColorScheme.dark(
    surface: surface,
    onSurface: onSurface,
    onSurfaceVariant: onSurfaceVariant,
    primary: primary,
    onPrimary: onPrimary,
    primaryContainer: primaryContainer,
    onPrimaryContainer: onPrimaryContainer,
    secondary: secondary,
    onSecondary: onSecondary,
    secondaryContainer: secondaryContainer,
    onSecondaryContainer: onSecondaryContainer,
    error: error,
    onError: onError,
    errorContainer: errorContainer,
    onErrorContainer: onErrorContainer,
    outline: outline,
    outlineVariant: outlineVariant,
    inverseSurface: onSurface,
    onInverseSurface: Color(0xFF392E26),
    surfaceContainerLowest: surfaceContainerLowest,
    surfaceContainerLow: surfaceContainerLow,
    surfaceContainer: surfaceContainer,
    surfaceContainerHigh: surfaceContainerHigh,
    surfaceContainerHighest: surfaceContainerHighest,
    surfaceTint: primary,
  );

  final geist = TextStyle(fontFamily: _geist, color: onSurface);
  final playfair = TextStyle(fontFamily: _playfair, color: onSurface);

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: surface,
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
      backgroundColor: surface,
      foregroundColor: onSurface,
      elevation: 0,
      titleTextStyle: playfair.copyWith(
        fontSize: 22,
        fontWeight: FontWeight.w700,
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: surfaceContainerLow,
      indicatorColor: Colors.transparent,
      iconTheme: WidgetStateProperty.resolveWith(
        (states) => IconThemeData(
          color: states.contains(WidgetState.selected)
              ? primary
              : onSurfaceVariant,
        ),
      ),
      labelTextStyle: WidgetStateProperty.resolveWith(
        (states) => TextStyle(
          fontFamily: _geist,
          fontSize: 12,
          fontWeight: FontWeight.w500,
          color: states.contains(WidgetState.selected)
              ? primary
              : onSurfaceVariant,
        ),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: surfaceContainerLow,
      hintStyle: geist.copyWith(color: outline),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: outlineVariant),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: outlineVariant),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: primary),
      ),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: surfaceContainerHigh,
      labelStyle: geist.copyWith(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        color: onSurfaceVariant,
      ),
      side: BorderSide(color: outlineVariant),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
    listTileTheme: ListTileThemeData(iconColor: onSurfaceVariant),
    cardTheme: CardThemeData(
      color: surfaceContainerLow,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: outlineVariant),
      ),
    ),
  );
}
