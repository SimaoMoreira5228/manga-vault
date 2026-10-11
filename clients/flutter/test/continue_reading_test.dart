import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/continue_reading.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _solo = ContinueItem(
  workId: 'w1',
  title: 'Solo Leveling',
  chapterId: 'c1',
  page: 3,
);

const _chainsaw = ContinueItem(
  workId: 'w2',
  title: 'Chainsaw Man',
  chapterId: 'c2',
  page: 1,
);

Future<void> _pump(WidgetTester tester, FakeVault vault) => tester.pumpWidget(
  MaterialApp(
    home: Scaffold(
      body: ContinueReadingRow(vault: vault, onChanged: () {}),
    ),
  ),
);

void main() {
  testWidgets('renders a resume tile per unread position', (tester) async {
    await _pump(tester, FakeVault(continueItems: const [_solo, _chainsaw]));
    await tester.pumpAndSettle();

    expect(find.text('CONTINUE READING'), findsOneWidget);
    expect(find.text('Solo Leveling'), findsOneWidget);
    expect(find.text('Chainsaw Man'), findsOneWidget);
  });

  testWidgets('dismissing a tile clears the stored page but keeps the work', (
    tester,
  ) async {
    final vault = FakeVault(continueItems: const [_solo, _chainsaw]);
    await _pump(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Clear reading position').first);
    await tester.pumpAndSettle();

    expect(vault.cleared, ['c1']);

    final after = await vault.continueReading();
    expect(after.map((item) => item.page), [isNull, 1]);
    expect(find.text('Solo Leveling'), findsOneWidget);
    expect(find.text('Chainsaw Man'), findsOneWidget);
  });

  testWidgets('reports a failed clear instead of dropping the tile', (
    tester,
  ) async {
    final vault = FakeVault(
      continueItems: const [_solo],
      clearError: 'position unavailable',
    );
    await _pump(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Clear reading position'));
    await tester.pumpAndSettle();

    expect(find.textContaining('Could not clear position'), findsOneWidget);
    expect(find.text('Solo Leveling'), findsOneWidget);
    expect(vault.cleared, isEmpty);
  });
}
