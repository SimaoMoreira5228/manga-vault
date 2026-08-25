import 'dart:io';

import 'package:flutter/material.dart';

import '../models/local_source.dart';

class LocalReader extends StatefulWidget {
  const LocalReader({super.key, required this.work, required this.chapters});

  final LocalWork work;
  final List<LocalChapter> chapters;

  @override
  State<LocalReader> createState() => _LocalReaderState();
}

class _LocalReaderState extends State<LocalReader> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final chapter = widget.chapters[_index];
    return Scaffold(
      appBar: AppBar(
        title: Text(chapter.title),
        actions: [
          IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: _index > 0 ? () => setState(() => _index--) : null,
          ),
          IconButton(
            icon: const Icon(Icons.arrow_forward),
            onPressed: _index < widget.chapters.length - 1
                ? () => setState(() => _index++)
                : null,
          ),
        ],
      ),
      body: _LocalChapterView(chapter: chapter),
    );
  }
}

class _LocalChapterView extends StatelessWidget {
  const _LocalChapterView({required this.chapter});
  final LocalChapter chapter;

  @override
  Widget build(BuildContext context) {
    if (chapter.isArchive) {
      final image = readChapterImage(chapter.path);
      if (image == null)
        return const Center(child: Text('Failed to read image'));
      return Center(child: Image.memory(image, fit: BoxFit.contain));
    }
    final dir = Directory(chapter.path);
    if (dir.existsSync()) {
      final images =
          dir
              .listSync()
              .whereType<File>()
              .where((f) => _isImage(f.path))
              .toList()
            ..sort((a, b) => a.path.compareTo(b.path));
      return ListView.builder(
        itemCount: images.length,
        itemBuilder: (context, index) =>
            Image.file(images[index], fit: BoxFit.cover),
      );
    }
    final file = File(chapter.path);
    if (file.existsSync() && _isImage(chapter.path)) {
      return Center(child: Image.file(file, fit: BoxFit.contain));
    }
    return const Center(child: Text('Unsupported format'));
  }

  bool _isImage(String path) {
    final lower = path.toLowerCase();
    return lower.endsWith('.jpg') ||
        lower.endsWith('.jpeg') ||
        lower.endsWith('.png') ||
        lower.endsWith('.webp') ||
        lower.endsWith('.gif');
  }
}
