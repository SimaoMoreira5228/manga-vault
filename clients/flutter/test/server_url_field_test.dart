import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/pages/server_url_field.dart';

void main() {
  test('adds a scheme when the user leaves it out', () {
    expect(normalizeServerUrl('127.0.0.1:8080'), 'http://127.0.0.1:8080');
    expect(normalizeServerUrl('vault.example.org'), 'http://vault.example.org');
  });

  test('keeps a scheme the user did type', () {
    expect(normalizeServerUrl('https://vault.example.org'), 'https://vault.example.org');
    expect(normalizeServerUrl('http://192.168.1.10:8080'), 'http://192.168.1.10:8080');
  });

  test('trims surrounding whitespace a paste brings along', () {
    expect(normalizeServerUrl('  127.0.0.1:8080\n'), 'http://127.0.0.1:8080');
  });

  test('drops a trailing slash and any path', () {
    expect(normalizeServerUrl('https://vault.example.org/'), 'https://vault.example.org');
    expect(normalizeServerUrl('https://vault.example.org/manga/'), 'https://vault.example.org');
  });

  test('rejects an empty address', () {
    expect(parseServerUrl(''), isNull);
    expect(parseServerUrl('   '), isNull);
    expect(serverUrlError(''), isNotNull);
  });

  test('rejects something that is not an address', () {
    expect(parseServerUrl('not a url at all'), isNull);
    expect(serverUrlError('not a url at all'), isNotNull);
  });

  test('accepts everything a real deployment looks like', () {
    for (final raw in [
      '127.0.0.1:8080',
      'http://localhost:5173',
      'https://manga.example.org',
      '192.168.0.5:8080',
    ]) {
      expect(parseServerUrl(raw), isNotNull, reason: raw);
      expect(serverUrlError(raw), isNull, reason: raw);
    }
  });
}
