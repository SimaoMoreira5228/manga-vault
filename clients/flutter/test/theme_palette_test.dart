import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/theme.dart';
import 'package:manga_vault/theme_palette_data.g.dart';

double _luminance(Color color) {
  double channel(double value) {
    final c = value / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) * ((c + 0.055) / 1.055) * (((c + 0.055) / 1.055));
  }

  return 0.2126 * channel(color.r * 255) +
      0.7152 * channel(color.g * 255) +
      0.0722 * channel(color.b * 255);
}

double contrastRatio(Color a, Color b) {
  final first = _luminance(a);
  final second = _luminance(b);
  final lighter = first > second ? first : second;
  final darker = first > second ? second : first;
  return (lighter + 0.05) / (darker + 0.05);
}

void main() {
  test('offers ten themes with distinct ids', () {
    expect(vaultThemes, hasLength(10));
    expect(vaultThemes.map((theme) => theme.id).toSet(), hasLength(10));
  });

  test('every theme id resolves back to its own theme', () {
    for (final theme in vaultThemes) {
      expect(vaultThemeById(theme.id).id, theme.id);
      expect(paletteForThemeId(theme.id).surface, theme.palette.surface);
    }
  });

  test('an unknown id falls back instead of throwing', () {
    expect(vaultThemeById('nope').id, vaultThemes.first.id);
  });

  test('surfaces are genuinely light or dark, matching the declared mode', () {
    for (final theme in vaultThemes) {
      final brightness = ThemeData.estimateBrightnessForColor(
        theme.palette.surface,
      );
      expect(
        brightness,
        theme.mode == VaultThemeMode.dark ? Brightness.dark : Brightness.light,
        reason: '${theme.id} is declared ${theme.mode.name} but reads as ${brightness.name}',
      );
    }
  });

  test('both modes are represented', () {
    expect(
      vaultThemes.where((theme) => theme.mode == VaultThemeMode.dark),
      isNotEmpty,
    );
    expect(
      vaultThemes.where((theme) => theme.mode == VaultThemeMode.light),
      isNotEmpty,
    );
  });

  test('body text clears WCAG AA on every surface it can land on', () {
    for (final theme in vaultThemes) {
      final palette = theme.palette;
      for (final background in [
        palette.surface,
        palette.surfaceLow,
        palette.surfaceContainer,
        palette.surfaceHigh,
        palette.surfaceHighest,
      ]) {
        expect(
          contrastRatio(palette.onSurface, background),
          greaterThanOrEqualTo(4.5),
          reason: 'onSurface on ${background.toARGB32().toRadixString(16)} in ${theme.id}',
        );
      }
    }
  });

  test('accent text clears WCAG AA on its own fill and on the surface', () {
    for (final theme in vaultThemes) {
      final palette = theme.palette;
      expect(
        contrastRatio(palette.onPrimary, palette.primary),
        greaterThanOrEqualTo(4.5),
        reason: theme.id,
      );
      expect(
        contrastRatio(palette.onPrimaryContainer, palette.primaryContainer),
        greaterThanOrEqualTo(4.5),
        reason: theme.id,
      );
      expect(
        contrastRatio(palette.primary, palette.surface),
        greaterThanOrEqualTo(3),
        reason: theme.id,
      );
    }
  });
}