import 'package:flutter/material.dart';

import '../theme.dart';

class AppearanceSection extends StatelessWidget {
  const AppearanceSection({super.key});

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<Appearance>(
      valueListenable: appearance,
      builder: (context, look, _) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
            child: Text(
              'APPEARANCE',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w500,
                letterSpacing: 0.6,
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final theme in vaultThemes)
                  SizedBox(
                    width: 104,
                    child: _ThemePickerCard(
                      theme: theme,
                      selected: theme.id == look.themeId,
                      onTap: () => applyAppearance(themeId: theme.id),
                    ),
                  ),
              ],
            ),
          ),
          SwitchListTile(
            title: const Text('Pure black background'),
            subtitle: Text(
              vaultThemeById(look.themeId).mode == VaultThemeMode.light
                  ? 'Not available on light themes.'
                  : 'Use #000 for the app background: ideal for OLED screens.',
            ),
            value:
                vaultThemeById(look.themeId).mode == VaultThemeMode.dark &&
                look.pureBlack,
            onChanged: (value) => applyAppearance(pureBlack: value),
          ),
        ],
      ),
    );
  }
}

class _ThemePickerCard extends StatelessWidget {
  const _ThemePickerCard({
    required this.theme,
    required this.selected,
    required this.onTap,
  });

  final VaultTheme theme;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final palette = theme.palette;
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: selected
                ? scheme.primary
                : scheme.outlineVariant.withValues(alpha: 0.5),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              height: 32,
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      theme.name,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                  if (selected)
                    Container(
                      width: 8,
                      height: 8,
                      decoration: BoxDecoration(
                        color: scheme.primary,
                        shape: BoxShape.circle,
                      ),
                    ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            AspectRatio(
              aspectRatio: 4 / 3,
              child: Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: palette.surface,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: palette.outlineVariant),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _bar(palette.primary, 4, 32),
                    const SizedBox(height: 6),
                    Expanded(
                      child: Row(
                        children: [
                          _block(palette.surfaceLow),
                          const SizedBox(width: 6),
                          _block(palette.surfaceContainer),
                        ],
                      ),
                    ),
                    const SizedBox(height: 6),
                    Opacity(opacity: 0.7, child: _bar(palette.primary, 4, 48)),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _bar(Color color, double height, double width) => Container(
    height: height,
    width: width,
    decoration: BoxDecoration(
      color: color,
      borderRadius: BorderRadius.circular(2),
    ),
  );

  Widget _block(Color color) => Expanded(
    child: DecoratedBox(
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(4),
      ),
    ),
  );
}
