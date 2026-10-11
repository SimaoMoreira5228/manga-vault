import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/service/vault_service.dart';

void main() {
  test('derives the window of a paginated read', () {
    final first = VaultPage<int>(
      items: const [1],
      total: 120,
      offset: 0,
      limit: 50,
    );
    expect(first.number, 1);
    expect(first.pages, 3);
    expect(first.hasPrevious, isFalse);
    expect(first.hasNext, isTrue);

    final last = VaultPage<int>(
      items: const [1],
      total: 120,
      offset: 100,
      limit: 50,
    );
    expect(last.number, 3);
    expect(last.hasNext, isFalse);
    expect(last.hasPrevious, isTrue);
  });
}
