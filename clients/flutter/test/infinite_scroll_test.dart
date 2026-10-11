import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/discover.dart';
import 'package:manga_vault/pages/history.dart';
import 'package:manga_vault/pages/library.dart';
import 'package:manga_vault/pages/sessions_section.dart';
import 'package:manga_vault/service/remote_service.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

LibraryItem _item(int index) => LibraryItem(
  entryId: 'e$index',
  work: WorkDetails(
    id: 'w$index',
    sourceId: 'katana',
    remoteUrl: 'https://s/$index',
    kind: 'manga',
    title: 'Title $index',
    authors: const [],
    genres: const [],
    chapters: const [],
  ),
);

HistoryEntry _read(int index) => HistoryEntry(
  readAt: null,
  chapterId: 'c$index',
  chapterTitle: 'Chapter $index',
  workId: 'w$index',
  workTitle: 'Work $index',
);

WorkSummary _work(int index) => WorkSummary(
  sourceId: 'katana',
  title: 'Trend $index',
  remoteUrl: 'https://s/$index',
);

SessionInfo _device(int index) => SessionInfo(
  token: 't$index',
  deviceLabel: 'Device $index',
  lastSeenAt: DateTime(2026, 1, 1),
);

List<LibraryItem> _library(int count) => [
  for (var index = 0; index < count; index += 1) _item(index),
];

List<HistoryEntry> _history(int count) => [
  for (var index = 0; index < count; index += 1) _read(index),
];

List<WorkSummary> _trending(int count) => [
  for (var index = 0; index < count; index += 1) _work(index),
];

List<SessionInfo> _devices(int count) => [
  for (var index = 0; index < count; index += 1) _device(index),
];

Future<void> _scrollToEnd(WidgetTester tester, Finder target) async {
  for (var round = 0; round < 2; round += 1) {
    await tester.drag(target, const Offset(0, -20000));
    await tester.pumpAndSettle();
  }
}

class FakeAccount extends RemoteService {
  FakeAccount(this.devices) : super(baseUrl: 'http://vault.test', token: 'me');

  final List<SessionInfo> devices;
  final requests = <int>[];

  @override
  Future<VaultPage<SessionInfo>> sessions({
    int offset = 0,
    int limit = pageSize,
  }) async {
    requests.add(offset);
    final end = offset + limit > devices.length
        ? devices.length
        : offset + limit;
    return VaultPage(
      items: offset >= devices.length ? const [] : devices.sublist(offset, end),
      total: devices.length,
      offset: offset,
      limit: limit,
    );
  }

  @override
  Future<void> revokeSession(String token) async {}
}

void main() {
  testWidgets('library asks for the next page only near the end', (
    tester,
  ) async {
    final vault = FakeVault(items: _library(60));
    await tester.pumpWidget(MaterialApp(home: LibraryPage(vault: vault)));
    await tester.pumpAndSettle();

    expect(vault.libraryOffsets, [0]);
    expect(find.text('Title 0'), findsOneWidget);

    await tester.drag(find.byType(CustomScrollView), const Offset(0, -300));
    await tester.pumpAndSettle();

    expect(vault.libraryOffsets, [0], reason: 'a short drag must not page');
    expect(find.text('Title 5'), findsOneWidget);
  });

  testWidgets('library appends the next page instead of replacing it', (
    tester,
  ) async {
    final vault = FakeVault(items: _library(60));
    await tester.pumpWidget(MaterialApp(home: LibraryPage(vault: vault)));
    await tester.pumpAndSettle();

    await _scrollToEnd(tester, find.byType(CustomScrollView));

    expect(vault.libraryOffsets, [0, 50]);
    expect(find.text('Title 59'), findsOneWidget);

    await tester.drag(find.byType(CustomScrollView), const Offset(0, 20000));
    await tester.pumpAndSettle();

    expect(find.text('Title 0'), findsOneWidget);
    expect(find.text('60 works'), findsOneWidget);
  });

  testWidgets('library stops once the accumulated total is reached', (
    tester,
  ) async {
    final vault = FakeVault(items: _library(60));
    await tester.pumpWidget(MaterialApp(home: LibraryPage(vault: vault)));
    await tester.pumpAndSettle();

    await _scrollToEnd(tester, find.byType(CustomScrollView));

    expect(vault.libraryOffsets, [0, 50]);
  });

  testWidgets('history appends the next page and then stops', (tester) async {
    final vault = FakeVault(reading: _history(60));
    await tester.pumpWidget(MaterialApp(home: HistoryPage(vault: vault)));
    await tester.pumpAndSettle();

    expect(vault.historyOffsets, [0]);
    expect(find.text('Work 0'), findsOneWidget);

    await _scrollToEnd(tester, find.byType(ListView));

    expect(vault.historyOffsets, [0, 50]);
    expect(find.text('Work 59'), findsOneWidget);

    await tester.drag(find.byType(ListView), const Offset(0, 20000));
    await tester.pumpAndSettle();

    expect(find.text('Work 0'), findsOneWidget);

    await _scrollToEnd(tester, find.byType(ListView));

    expect(vault.historyOffsets, [0, 50]);
  });

  testWidgets('trending appends the next page and stops on an empty one', (
    tester,
  ) async {
    const source = SourceSummary(
      id: 'katana',
      name: 'Katana',
      version: '1.0.0',
      kind: 'manga',
    );
    final vault = FakeVault(sources: const [source], trending: _trending(60));
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SourceResults(vault: vault, sourceId: 'katana', onOpen: (_) {}),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await _scrollToEnd(tester, find.byType(GridView));

    expect(vault.latestRequests, [
      (sourceId: 'katana', page: 1),
      (sourceId: 'katana', page: 2),
      (sourceId: 'katana', page: 3),
    ]);
    await tester.drag(find.byType(GridView), const Offset(0, 20000));
    await tester.pumpAndSettle();
    expect(find.text('Trend 0'), findsOneWidget, reason: 'page 1 survives the append');
  });

  testWidgets('a failed page reports itself and retries on demand', (
    tester,
  ) async {
    const source = SourceSummary(
      id: 'katana',
      name: 'Katana',
      version: '1.0.0',
      kind: 'manga',
    );
    final vault = FakeVault(sources: const [source], trending: _trending(60));
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SourceResults(vault: vault, sourceId: 'katana', onOpen: (_) {}),
        ),
      ),
    );
    await tester.pumpAndSettle();

    vault.latestError = 'source offline';
    await _scrollToEnd(tester, find.byType(GridView));

    expect(find.textContaining('Could not load more'), findsOneWidget);
    expect(vault.latestRequests.last.page, 2);

    vault.latestError = null;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.textContaining('Could not load more'), findsNothing);
    expect(find.text('Trend 50'), findsOneWidget);
  });

  testWidgets('devices append the next page inside the settings scroll', (
    tester,
  ) async {
    final account = FakeAccount(_devices(60));
    final key = GlobalKey<SessionsSectionState>();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: NotificationListener<ScrollNotification>(
            onNotification: (notification) {
              key.currentState?.loadMore(notification);
              return false;
            },
            child: SingleChildScrollView(
              child: SessionsSection(
                key: key,
                account: account,
                onSignedOut: () {},
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(account.requests, [0]);
    expect(find.text('Device 0'), findsOneWidget);

    await _scrollToEnd(tester, find.byType(SingleChildScrollView));

    expect(account.requests, [0, 50]);
    expect(find.text('Device 59'), findsOneWidget);

    await _scrollToEnd(tester, find.byType(SingleChildScrollView));

    expect(account.requests, [0, 50]);
  });
}
