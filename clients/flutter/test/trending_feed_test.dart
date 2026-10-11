import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/discover.dart';
import 'package:manga_vault/pages/trending_feed.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _katana = SourceSummary(
  id: 'katana',
  name: 'Katana',
  version: '1.0.0',
  kind: 'manga',
);

const _novelfire = SourceSummary(
  id: 'novelfire',
  name: 'NovelFire',
  version: '2.0.0',
  kind: 'novel',
);

WorkSummary _work(String title) =>
    WorkSummary(sourceId: _katana.id, title: title, remoteUrl: 'https://s/$title');

Future<void> _pumpDiscover(WidgetTester tester, FakeVault vault) =>
    tester.pumpWidget(MaterialApp(home: DiscoverPage(vault: vault)));

void main() {
  testWidgets('leads discover with a ranked trending tab', (tester) async {
    final vault = FakeVault(
      sources: const [_katana, _novelfire],
      trending: [_work('First'), _work('Second')],
    );
    await _pumpDiscover(tester, vault);
    await tester.pumpAndSettle();

    expect(find.widgetWithText(Tab, 'Trending'), findsOneWidget);
    expect(find.widgetWithText(Tab, 'All'), findsOneWidget);
    expect(vault.trendingRequests, [
      (sourceId: 'katana', page: 1),
    ]);
    expect(find.text('1'), findsOneWidget);
    expect(find.text('2'), findsOneWidget);
    expect(find.text('First'), findsOneWidget);
    expect(find.text('manga'), findsNWidgets(2));
  });

  testWidgets('appends the next trending page when the list runs out', (
    tester,
  ) async {
    final vault = FakeVault(
      sources: const [_katana, _novelfire],
      trending: [_work('First')],
    );
    await _pumpDiscover(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.widgetWithText(ChoiceChip, 'NovelFire'));
    await tester.pumpAndSettle();
    expect(find.text('novel'), findsOneWidget);

    expect(vault.trendingRequests, [
      (sourceId: 'katana', page: 1),
      (sourceId: 'novelfire', page: 1),
    ]);
    expect(find.text('First'), findsOneWidget);
  });

  testWidgets('reports a trending failure and retries on demand', (
    tester,
  ) async {
    final vault = FakeVault(
      sources: const [_katana],
      trending: [_work('First')],
      trendingError: 'source offline',
    );
    await _pumpDiscover(tester, vault);
    await tester.pumpAndSettle();

    expect(find.text('Could not load trending'), findsOneWidget);

    vault.trendingError = null;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('First'), findsOneWidget);
    expect(vault.trendingRequests, [
      (sourceId: 'katana', page: 1),
      (sourceId: 'katana', page: 1),
    ]);
  });

  testWidgets('hands a tapped result to the opener with its source', (
    tester,
  ) async {
    final opened = <String>[];
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: TrendingFeed(
            vault: FakeVault(
              sources: const [_katana],
              trending: [_work('Solo Leveling')],
            ),
            sources: const [_katana],
            onOpen: (sourceId, work) => opened.add('$sourceId/${work.title}'),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Solo Leveling'));

    expect(opened, ['katana/Solo Leveling']);
  });
}