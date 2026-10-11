import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/library.dart';
import 'package:manga_vault/service/vault_service.dart';

import 'fake_vault.dart';

const _shonen = CategoryInfo(id: 'c1', name: 'Shonen');

Future<void> _pumpLibrary(WidgetTester tester, FakeVault vault) =>
    tester.pumpWidget(MaterialApp(home: LibraryPage(vault: vault)));

void main() {
  testWidgets('creates a category from the library filter row', (tester) async {
    final vault = FakeVault();
    await _pumpLibrary(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.text('New'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.byType(TextField),
      ),
      'Reading',
    );
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();

    expect(vault.created, ['Reading']);
    expect(find.text('Reading'), findsOneWidget);
  });

  testWidgets('ignores an empty category name', (tester) async {
    final vault = FakeVault();
    await _pumpLibrary(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.text('New'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();

    expect(vault.created, isEmpty);
  });

  testWidgets('deletes a category from its chip', (tester) async {
    final vault = FakeVault(categories: const [_shonen]);
    await _pumpLibrary(tester, vault);
    await tester.pumpAndSettle();

    expect(find.text('Shonen'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.close));
    await tester.pumpAndSettle();

    expect(vault.deleted, ['c1']);
    expect(find.text('Shonen'), findsNothing);
  });

  testWidgets('selects a category to filter the library, and toggles it off', (
    tester,
  ) async {
    final vault = FakeVault(categories: const [_shonen]);
    await _pumpLibrary(tester, vault);
    await tester.pumpAndSettle();

    await tester.tap(find.text('Shonen'));
    await tester.pumpAndSettle();
    expect(vault.filters.last.categoryId, 'c1');

    await tester.tap(find.text('Shonen'));
    await tester.pumpAndSettle();
    expect(vault.filters.last.categoryId, isNull);
  });
}