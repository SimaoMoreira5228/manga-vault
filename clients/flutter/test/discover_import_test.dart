import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/discover.dart';
import 'package:manga_vault/pages/work_page.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _source = SourceSummary(
  id: 'katana',
  name: 'Katana',
  version: '1.0.0',
  kind: 'manga',
);

List<WorkSummary> _trending(int count) => [
  for (var index = 0; index < count; index += 1)
    WorkSummary(
      sourceId: 'katana',
      title: 'Trend $index',
      remoteUrl: 'https://s/$index',
    ),
];

FakeVault _vault({Completer<void>? gate}) {
  final vault = FakeVault(sources: const [_source], trending: _trending(6));
  vault.importGate = gate;
  return vault;
}

Future<void> _pumpDiscover(WidgetTester tester, FakeVault vault) async {
  await tester.pumpWidget(MaterialApp(home: DiscoverPage(vault: vault)));
  await tester.pumpAndSettle();
}

Future<void> _tapWork(WidgetTester tester) async {
  await tester.tap(find.text('Trend 0'));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

void main() {
  testWidgets('importing shows a non-dismissable progress dialog that blocks', (
    tester,
  ) async {
    final gate = Completer<void>();
    final vault = _vault(gate: gate);
    await _pumpDiscover(tester, vault);

    await _tapWork(tester);

    expect(find.byType(AlertDialog), findsOneWidget);
    expect(find.text('Importing chapters'), findsOneWidget);
    expect(
      find.text('This can take a minute on a long series.'),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.byType(CircularProgressIndicator),
      ),
      findsOneWidget,
    );
    final barrier = tester.widget<ModalBarrier>(find.byType(ModalBarrier).last);
    expect(barrier.dismissible, isFalse);

    await tester.tapAt(const Offset(20, 20));
    await tester.pump();
    await tester.tap(find.text('All'), warnIfMissed: false);
    await tester.pump();
    await tester.tap(find.text('Trend 1'), warnIfMissed: false);
    await tester.pump();

    expect(find.byType(AlertDialog), findsOneWidget);
    expect(find.byType(AggregatedSearch), findsNothing);
    expect(vault.imports, hasLength(1));
    expect(vault.trendingRequests, hasLength(1));

    gate.complete();
    await tester.pumpAndSettle();

    expect(find.byType(AlertDialog), findsNothing);
  });

  testWidgets('a finished import closes the dialog and opens the work', (
    tester,
  ) async {
    final gate = Completer<void>();
    final vault = _vault(gate: gate);
    await _pumpDiscover(tester, vault);

    await _tapWork(tester);
    expect(find.byType(AlertDialog), findsOneWidget);

    gate.complete();
    await tester.pumpAndSettle();

    expect(find.byType(AlertDialog), findsNothing);
    expect(find.byType(WorkPage), findsOneWidget);
    expect(find.text('Imported Work'), findsWidgets);
    expect(vault.imports, [(sourceId: 'katana', remoteUrl: 'https://s/0')]);
  });

  testWidgets('a failed import closes the dialog and reports the error', (
    tester,
  ) async {
    final gate = Completer<void>();
    final vault = _vault(gate: gate);
    vault.importError = 'source offline';
    await _pumpDiscover(tester, vault);

    await _tapWork(tester);
    expect(find.byType(AlertDialog), findsOneWidget);

    gate.complete();
    await tester.pumpAndSettle();

    expect(find.byType(AlertDialog), findsNothing);
    expect(find.byType(WorkPage), findsNothing);
    expect(find.text('Import failed: source offline'), findsOneWidget);
  });
}
