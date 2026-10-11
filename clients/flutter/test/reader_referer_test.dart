import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/reader_page.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _chapters = [
  ChapterSummary(id: 'c1', title: 'Chapter 1', sortIndex: 0),
];

const _work = WorkDetails(
  id: 'w1',
  sourceId: 'freewebnovel',
  remoteUrl: 'https://freewebnovel.com/novel/shadow-slave',
  kind: 'novel',
  title: 'Shadow Slave',
  authors: [],
  genres: [],
  chapters: [],
);

FakeVault _readerVault() => FakeVault(
  works: {'w1': _work},
  contents: {
    'c1': const ChapterBody.images([
      'https://freewebnovel.com/files/article/image/1/1991/1991s.jpg',
    ]),
  },
);

Widget _host(VaultService vault) => MaterialApp(
  home: ReaderPage(
    vault: vault,
    chapters: _chapters,
    index: 0,
    workId: 'w1',
  ),
);

void main() {
  testWidgets('never asks the proxy for an image without a referer', (
    tester,
  ) async {
    final vault = _readerVault();
    await tester.pumpWidget(_host(vault));
    await tester.pumpAndSettle();

    expect(vault.imageUrls, isNotEmpty, reason: 'the chapter image is rendered');
    expect(find.byType(Image), findsWidgets);

    final urls = tester
        .widgetList<Image>(find.byType(Image))
        .map((image) => (image.image as NetworkImage).url)
        .toList();
    expect(urls, isNotEmpty);
    for (final url in urls) {
      expect(url, contains('referer='), reason: '$url would be rejected with a 400');
    }
  });

  testWidgets('uses the work url as the referer', (tester) async {
    final vault = _readerVault();
    await tester.pumpWidget(_host(vault));
    await tester.pumpAndSettle();

    final urls = tester
        .widgetList<Image>(find.byType(Image))
        .map((image) => (image.image as NetworkImage).url)
        .toList();
    expect(
      urls.any((url) => url.contains('referer=https://freewebnovel.com/novel/shadow-slave')),
      isTrue,
      reason: 'the referer must be the work url so the proxy can resolve a plugin',
    );
  });
}
