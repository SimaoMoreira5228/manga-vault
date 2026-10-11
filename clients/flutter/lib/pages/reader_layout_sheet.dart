import 'package:flutter/material.dart';

class ReaderSettings {
  const ReaderSettings({
    required this.imageMarginPct,
    required this.imageGapPct,
    required this.fontSize,
    required this.lineHeight,
    required this.pagedMode,
    required this.rtlMode,
  });

  final double imageMarginPct;
  final double imageGapPct;
  final double fontSize;
  final double lineHeight;
  final bool pagedMode;
  final bool rtlMode;
}

class ReaderLayoutSheet extends StatefulWidget {
  const ReaderLayoutSheet({
    super.key,
    required this.kind,
    required this.settings,
    required this.onChange,
  });

  final String? kind;
  final ReaderSettings settings;
  final void Function(ReaderSettings settings) onChange;

  bool get showsText => kind != 'manga';

  bool get showsImages => kind != 'novel';

  @override
  State<ReaderLayoutSheet> createState() => _ReaderLayoutSheetState();
}

class _ReaderLayoutSheetState extends State<ReaderLayoutSheet> {
  late ReaderSettings value = widget.settings;

  void _set(ReaderSettings next) {
    setState(() => value = next);
    widget.onChange(next);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Center(
              child: Container(
                width: 36,
                height: 4,
                decoration: BoxDecoration(
                  color: theme.colorScheme.outlineVariant,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
            const SizedBox(height: 18),
            if (widget.showsText) ...[
              _Heading('Text'),
              _Slider(
                label: 'Font size',
                value: value.fontSize,
                min: 10,
                max: 34,
                divisions: 24,
                unit: 'px',
                onChanged: (next) => _set(_copy(value, fontSize: next)),
              ),
              _Slider(
                label: 'Line spacing',
                value: value.lineHeight,
                min: 1.1,
                max: 2.6,
                divisions: 15,
                unit: '',
                decimals: 1,
                onChanged: (next) => _set(_copy(value, lineHeight: next)),
              ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Right to left'),
                value: value.rtlMode,
                onChanged: (next) => _set(_copy(value, rtlMode: next)),
              ),
            ],
            if (widget.showsImages) ...[
              if (widget.showsText) const SizedBox(height: 12),
              _Heading('Images'),
              _Slider(
                label: 'Side margins',
                value: value.imageMarginPct,
                max: 50,
                divisions: 50,
                unit: '%',
                onChanged: (next) =>
                    _set(_copy(value, imageMarginPct: next)),
              ),
              _Slider(
                label: 'Gap between images',
                value: value.imageGapPct,
                max: 25,
                divisions: 25,
                unit: '%',
                onChanged: (next) => _set(_copy(value, imageGapPct: next)),
              ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Paged mode'),
                subtitle: const Text('One page at a time instead of scrolling'),
                value: value.pagedMode,
                onChanged: (next) => _set(_copy(value, pagedMode: next)),
              ),
            ],
          ],
        ),
      ),
    );
  }

  ReaderSettings _copy(
    ReaderSettings base, {
    double? fontSize,
    double? lineHeight,
    double? imageMarginPct,
    double? imageGapPct,
    bool? pagedMode,
    bool? rtlMode,
  }) => ReaderSettings(
    fontSize: fontSize ?? base.fontSize,
    lineHeight: lineHeight ?? base.lineHeight,
    imageMarginPct: imageMarginPct ?? base.imageMarginPct,
    imageGapPct: imageGapPct ?? base.imageGapPct,
    pagedMode: pagedMode ?? base.pagedMode,
    rtlMode: rtlMode ?? base.rtlMode,
  );
}

class _Heading extends StatelessWidget {
  const _Heading(this.text);

  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 10),
    child: Text(
      text.toUpperCase(),
      style: Theme.of(context).textTheme.labelSmall?.copyWith(
        color: Theme.of(context).colorScheme.onSurfaceVariant,
        letterSpacing: 0.8,
      ),
    ),
  );
}

class _Slider extends StatelessWidget {
  const _Slider({
    required this.label,
    required this.value,
    required this.max,
    required this.divisions,
    required this.unit,
    required this.onChanged,
    this.min = 0,
    this.decimals = 0,
  });

  final String label;
  final double value;
  final double min;
  final double max;
  final int divisions;
  final String unit;
  final int decimals;
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    final shown = value.toStringAsFixed(decimals);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: Theme.of(context).textTheme.bodyMedium),
            Text(
              '$shown$unit',
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
        Slider(
          value: value.clamp(min, max),
          min: min,
          max: max,
          divisions: divisions,
          label: '$shown$unit',
          onChanged: onChanged,
        ),
      ],
    );
  }
}