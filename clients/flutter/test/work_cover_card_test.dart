import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/work_cover_card.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

Widget _host(Widget child) =>
    MaterialApp(home: Scaffold(body: SizedBox(width: 400, child: child)));

void main() {
  testWidgets('shows the title, source and kind on the card', (tester) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Disastrous Necromancer',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/x',
          kind: 'manga',
          sourceId: 'hari_manga',
        ),
      ),
    );

    expect(find.text('Disastrous Necromancer'), findsOneWidget);
    expect(find.text('hari_manga'), findsOneWidget);
    expect(find.text('manga'), findsOneWidget);
  });

  testWidgets('renders the title exactly once when there is no cover', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Untitled',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/x',
        ),
      ),
    );
    expect(find.text('Untitled'), findsOneWidget);
  });

  testWidgets('sends the work url as the image referer', (tester) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'With Cover',
          coverUrl: 'https://cdn.test/cover.jpg',
          remoteUrl: 'https://example.test/manga/with-cover',
        ),
      ),
    );

    final image = tester.widget<Image>(find.byType(Image));
    expect(image.image as NetworkImage, isA<NetworkImage>());
    expect((image.image as NetworkImage).url, contains('referer='));
    expect((image.image as NetworkImage).url, contains('with-cover'));
  });

  testWidgets('omits the referer when the work has no remote url', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'No Remote',
          coverUrl: 'https://cdn.test/cover.jpg',
          remoteUrl: '',
        ),
      ),
    );
    final image = tester.widget<Image>(find.byType(Image));
    expect((image.image as NetworkImage).url, isNot(contains('referer=')));
  });

  testWidgets('a broken cover falls back instead of showing an error box', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Broken Cover',
          coverUrl: 'https://cdn.test/gone.jpg',
          remoteUrl: 'https://example.test/manga/broken',
        ),
      ),
    );

    final image = tester.widget<Image>(find.byType(Image));
    expect(image.errorBuilder, isNotNull);
  });

  testWidgets('shows unread progress only when there is unread work', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Reading',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/r',
          progress: const ChapterProgressStats(read: 3, total: 10),
        ),
      ),
    );
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
    expect(find.text('7'), findsOneWidget);

    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Caught Up',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/c',
          progress: const ChapterProgressStats(read: 10, total: 10),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('0'), findsNothing);
  });

  testWidgets('announces one label instead of reading every child', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Solo Leveling',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/s',
          kind: 'manga',
          progress: const ChapterProgressStats(read: 1, total: 12),
          onTap: () {},
        ),
      ),
    );

    final node = tester.getSemantics(find.byType(WorkCoverCard));
    expect(node.label, 'Solo Leveling, manga, 11 unread');
  });

  testWidgets('taps and long presses are wired through', (tester) async {
    var taps = 0;
    var holds = 0;
    await tester.pumpWidget(
      _host(
        WorkCoverCard(
          vault: FakeVault(),
          title: 'Tappable',
          coverUrl: null,
          remoteUrl: 'https://example.test/manga/t',
          onTap: () => taps += 1,
          onLongPress: () => holds += 1,
        ),
      ),
    );

    await tester.tap(find.byType(WorkCoverCard));
    await tester.longPress(find.byType(WorkCoverCard));
    expect(taps, 1);
    expect(holds, 1);
  });
}
