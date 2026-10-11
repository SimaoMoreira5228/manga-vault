import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/source_page_bar.dart';

Finder _button(String label) => find.widgetWithText(OutlinedButton, label);

bool _enabled(WidgetTester tester, String label) =>
    tester.widget<OutlinedButton>(_button(label)).onPressed != null;

void main() {
  testWidgets('blocks Previous on the first page and reports the page number', (
    tester,
  ) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(body: SourcePageBar(page: 1, canAdvance: true, onshift: _ignore)),
      ),
    );

    expect(find.text('Page 1'), findsOneWidget);
    expect(_enabled(tester, 'Previous'), isFalse);
    expect(_enabled(tester, 'Next'), isTrue);
  });

  testWidgets('steps both directions and honours the end-of-results signal', (
    tester,
  ) async {
    final shifts = <int>[];
    Widget bar(int page, bool canAdvance) => MaterialApp(
      home: Scaffold(
        body: SourcePageBar(page: page, canAdvance: canAdvance, onshift: shifts.add),
      ),
    );

    await tester.pumpWidget(bar(2, true));
    await tester.tap(_button('Next'));
    await tester.tap(_button('Previous'));
    await tester.pumpWidget(bar(2, false));

    expect(shifts, [3, 1]);
    expect(_enabled(tester, 'Next'), isFalse);
  });
}

void _ignore(int _) {}