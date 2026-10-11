import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/stats.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _summary = ReadingStatsSummary(
  totalRead: 12345,
  streak: 4,
  worksStarted: 7,
  daily: {'2026-10-01': 2, '2026-10-02': 5, '2026-10-03': 1},
);

const _empty = ReadingStatsSummary(
  totalRead: 0,
  streak: 0,
  worksStarted: 0,
  daily: {},
);

Future<void> _pump(WidgetTester tester, FakeVault vault) => tester.pumpWidget(
  MaterialApp(home: StatsPage(vault: vault)),
);

void main() {
  testWidgets('renders the reading summary and the daily chart', (
    tester,
  ) async {
    await _pump(tester, FakeVault(stats: _summary));
    await tester.pumpAndSettle();

    expect(find.text('Chapters read'), findsOneWidget);
    expect(find.text('12,345'), findsOneWidget);
    expect(find.text('Reading streak'), findsOneWidget);
    expect(find.text('4 days'), findsOneWidget);
    expect(find.text('Works started'), findsOneWidget);
    expect(find.text('7'), findsOneWidget);
    expect(find.text('Last 30 days'), findsOneWidget);
  });

  testWidgets('hides the daily chart when nothing was read', (tester) async {
    await _pump(tester, FakeVault(stats: _empty));
    await tester.pumpAndSettle();

    expect(find.text('Last 30 days'), findsNothing);
    expect(find.text('0'), findsWidgets);
  });

  testWidgets('reports a failure and recovers when retrying', (tester) async {
    final vault = FakeVault(stats: _summary, statsError: 'stats unavailable');
    await _pump(tester, vault);
    await tester.pumpAndSettle();

    expect(find.text('Could not load statistics'), findsOneWidget);
    expect(find.text('stats unavailable'), findsOneWidget);

    vault.statsError = null;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('Could not load statistics'), findsNothing);
    expect(find.text('12,345'), findsOneWidget);
  });
}