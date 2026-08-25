import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';

class LocalWork {
  const LocalWork({required this.title, required this.path});
  final String title;
  final String path;
}

class LocalChapter {
  const LocalChapter({
    required this.title,
    required this.path,
    required this.isArchive,
  });
  final String title;
  final String path;
  final bool isArchive;
}

List<LocalWork> scanLibrary(String rootPath) {
  final root = Directory(rootPath);
  if (!root.existsSync()) return [];
  final works = <LocalWork>[];
  for (final entity in root.listSync()) {
    if (entity is Directory && !entity.path.endsWith('/.')) {
      works.add(
        LocalWork(
          title: entity.path.split(Platform.pathSeparator).last,
          path: entity.path,
        ),
      );
    } else if (entity is File && entity.path.toLowerCase().endsWith('.cbz')) {
      final name = entity.path.split(Platform.pathSeparator).last;
      works.add(
        LocalWork(title: name.replaceAll('.cbz', ''), path: entity.path),
      );
    }
  }
  works.sort((a, b) => a.title.compareTo(b.title));
  return works;
}

List<LocalChapter> chaptersForWork(LocalWork work) {
  final file = File(work.path);
  if (file.existsSync()) {
    return _chaptersFromCbz(work.path);
  }
  return _chaptersFromFolder(Directory(work.path));
}

List<LocalChapter> _chaptersFromCbz(String cbzPath) {
  final bytes = File(cbzPath).readAsBytesSync();
  final archive = ZipDecoder().decodeBytes(bytes);
  final chapters = <LocalChapter>[];
  for (final entry in archive) {
    if (entry.isFile && _isImage(entry.name)) {
      chapters.add(
        LocalChapter(
          title: entry.name,
          path: '$cbzPath::${entry.name}',
          isArchive: true,
        ),
      );
    }
  }
  chapters.sort((a, b) => a.title.compareTo(b.title));
  return chapters;
}

List<LocalChapter> _chaptersFromFolder(Directory dir) {
  final chapters = <LocalChapter>[];
  if (!dir.existsSync()) return chapters;
  for (final entity in dir.listSync()) {
    if (entity is File && _isImage(entity.path)) {
      chapters.add(
        LocalChapter(
          title: entity.path.split(Platform.pathSeparator).last,
          path: entity.path,
          isArchive: false,
        ),
      );
    } else if (entity is Directory) {
      chapters.add(
        LocalChapter(
          title: entity.path.split(Platform.pathSeparator).last,
          path: entity.path,
          isArchive: false,
        ),
      );
    }
  }
  chapters.sort((a, b) => a.title.compareTo(b.title));
  return chapters;
}

Uint8List? readChapterImage(String path) {
  if (path.contains('::')) {
    final parts = path.split('::');
    final cbzPath = parts[0];
    final entryName = parts.sublist(1).join('::');
    final bytes = File(cbzPath).readAsBytesSync();
    final archive = ZipDecoder().decodeBytes(bytes);
    for (final entry in archive) {
      if (entry.isFile && entry.name == entryName) {
        return entry.content as Uint8List?;
      }
    }
    return null;
  }
  final file = File(path);
  if (!file.existsSync()) return null;
  return file.readAsBytesSync();
}

bool _isImage(String path) {
  final lower = path.toLowerCase();
  return lower.endsWith('.jpg') ||
      lower.endsWith('.jpeg') ||
      lower.endsWith('.png') ||
      lower.endsWith('.webp') ||
      lower.endsWith('.gif');
}
