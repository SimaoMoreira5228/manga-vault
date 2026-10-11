import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/reader_layout_sheet.dart';

const _defaults = ReaderSettings(
  imageMarginPct: 8,
  imageGapPct: 4,
  fontSize: 16,
  lineHeight: 1.6,
  pagedMode: false,
  rtlMode: false,
);

Widget _host({
  String? kind = 'novel',
  ReaderSettings? settings,
  void Function(ReaderSettings)? onChange,
}) => MaterialApp(
  home: Scaffold(
    body: ReaderLayoutSheet(
      kind: kind,
      settings: settings ?? _defaults,
      onChange: onChange ?? (_) {},
    ),
  ),
);

void main() {
  testWidgets('a novel offers text settings only', (tester) async {
    await tester.pumpWidget(_host(kind: 'novel'));

    expect(find.text('TEXT'), findsOneWidget);
    expect(find.text('Font size'), findsOneWidget);
    expect(find.text('Line spacing'), findsOneWidget);
    expect(find.text('Right to left'), findsOneWidget);
    expect(find.text('IMAGES'), findsNothing);
    expect(find.text('Side margins'), findsNothing);
    expect(find.text('Paged mode'), findsNothing);
  });

  testWidgets('a manga offers image settings only', (tester) async {
    await tester.pumpWidget(_host(kind: 'manga'));

    expect(find.text('IMAGES'), findsOneWidget);
    expect(find.text('Side margins'), findsOneWidget);
    expect(find.text('Gap between images'), findsOneWidget);
    expect(find.text('Paged mode'), findsOneWidget);
    expect(find.text('Font size'), findsNothing);
    expect(find.text('Right to left'), findsNothing);
  });

  testWidgets('an unknown kind offers everything', (tester) async {
    await tester.pumpWidget(_host(kind: null));

    expect(find.text('Font size'), findsOneWidget);
    expect(find.text('Side margins'), findsOneWidget);
    expect(find.byType(Slider), findsNWidgets(4));
  });

  testWidgets('margins are percentages, not pixels', (tester) async {
    await tester.pumpWidget(
      _host(
        kind: 'manga',
        settings: const ReaderSettings(
          imageMarginPct: 12,
          imageGapPct: 6,
          fontSize: 16,
          lineHeight: 1.6,
          pagedMode: false,
          rtlMode: false,
        ),
      ),
    );

    expect(find.text('12%'), findsOneWidget);
    expect(find.text('6%'), findsOneWidget);
    expect(find.textContaining('px'), findsNothing);
  });

  testWidgets('margins can be pushed until the image is gone', (tester) async {
    await tester.pumpWidget(_host(kind: 'manga'));

    final margin = tester.widget<Slider>(find.byType(Slider).first);
    expect(margin.max, 50, reason: '50% each side leaves the image no width');

    final gap = tester.widget<Slider>(find.byType(Slider).at(1));
    expect(gap.max, 25);
  });

  testWidgets('every control reports the new value', (tester) async {
    final changes = <ReaderSettings>[];
    await tester.pumpWidget(
      _host(kind: 'novel', onChange: changes.add),
    );

    await tester.drag(find.byType(Slider).first, const Offset(60, 0));
    await tester.pumpAndSettle();

    expect(changes, isNotEmpty);
    expect(changes.last.fontSize, greaterThan(16));
  });

  testWidgets('toggling right to left is reported', (tester) async {
    final changes = <ReaderSettings>[];
    await tester.pumpWidget(_host(kind: 'novel', onChange: changes.add));

    await tester.tap(find.byType(SwitchListTile));
    await tester.pumpAndSettle();

    expect(changes.last.rtlMode, isTrue);
  });
}