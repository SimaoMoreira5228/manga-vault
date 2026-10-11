import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/theme.dart';

void main() {
  test('defaults to a dark theme', () {
    expect(darkDefaultThemeId, 'classic');
    expect(
      vaultThemeById(darkDefaultThemeId).mode,
      VaultThemeMode.dark,
    );
  });

  test('falls back to the default when nothing is stored', () {
    expect(resolveThemeId(null), darkDefaultThemeId);
  });

  test('falls back to the default when the stored id is not a theme', () {
    expect(resolveThemeId('sunset'), darkDefaultThemeId);
    expect(resolveThemeId(''), darkDefaultThemeId);
  });

  test('keeps a stored choice', () {
    for (final theme in vaultThemes) {
      expect(resolveThemeId(theme.id), theme.id);
    }
  });

  test('resolving to a light theme drops pure black', () {
    final onLight = resolveAppearance(
      storedThemeId: 'sakura',
      pureBlack: true,
    );
    expect(onLight.themeId, 'sakura');
    expect(onLight.pureBlack, isFalse);

    final onDark = resolveAppearance(
      storedThemeId: 'forest',
      pureBlack: true,
    );
    expect(onDark.pureBlack, isTrue);
  });
}