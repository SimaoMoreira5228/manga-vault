import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('every remote image url carries a referer', () {
    final callSites = <String>[];
    for (final file in Directory('lib').listSync(recursive: true).whereType<File>()) {
      if (!file.path.endsWith('.dart')) continue;
      final lines = file.readAsLinesSync();
      for (var index = 0; index < lines.length; index += 1) {
        final line = lines[index];
        if (!line.contains('imageUrl(') || line.contains('String imageUrl')) {
          continue;
        }
        if (!line.contains('referer:')) {
          callSites.add('${file.path}:${index + 1}');
        }
      }
    }

    expect(
      callSites,
      isEmpty,
      reason: 'the server answers 400 without a referer, so the cover never loads',
    );
  });

  test('the guard reads both a qualified and an unqualified call', () {
    expect('widget.vault.imageUrl(item.coverUrl!),'.contains('imageUrl('), isTrue);
    expect('imageUrl(details.coverUrl!),'.contains('imageUrl('), isTrue);
    expect('String imageUrl(String url)'.contains('String imageUrl'), isTrue);
  });
}
