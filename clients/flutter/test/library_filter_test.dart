import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/service/library_filter.dart';
import 'package:manga_vault/src/rust/api/local.dart';

void main() {
  test('appends only the filters that are set', () {
    expect(libraryQuerySuffix(const LibraryFilter()), '');
    expect(
      libraryQuerySuffix(const LibraryFilter(kind: 'manga')),
      '&kind=manga',
    );
    expect(
      libraryQuerySuffix(
        const LibraryFilter(kind: 'novel', categoryId: 'c1', query: '  a b '),
      ),
      '&kind=novel&category_id=c1&q=a+b',
    );
    expect(libraryQuerySuffix(const LibraryFilter(query: '   ')), '');
  });
}
