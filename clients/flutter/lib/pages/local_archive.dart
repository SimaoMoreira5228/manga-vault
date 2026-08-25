import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';

import '../models/local_source.dart';
import '../service/app_prefs.dart';
import 'local_reader.dart';

class LocalArchivePage extends StatefulWidget {
  const LocalArchivePage({super.key});

  @override
  State<LocalArchivePage> createState() => _LocalArchivePageState();
}

class _LocalArchivePageState extends State<LocalArchivePage> {
  List<String> _paths = [];
  List<LocalWork> _works = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final prefs = await AppPrefs.instance();
    _paths = prefs.localLibraryPaths;
    final allWorks = <LocalWork>[];
    for (final path in _paths) {
      allWorks.addAll(scanLibrary(path));
    }
    if (mounted) setState(() => _works = allWorks);
  }

  Future<void> _addPath() async {
    final result = await FilePicker.getDirectoryPath(
      dialogTitle: 'Select manga/novel folder',
    );
    if (result == null) return;
    final prefs = await AppPrefs.instance();
    await prefs.addLocalLibraryPath(result);
    await _load();
  }

  Future<void> _removePath(String path) async {
    final prefs = await AppPrefs.instance();
    await prefs.removeLocalLibraryPath(path);
    await _load();
  }

  void _openWork(LocalWork work) {
    final chapters = chaptersForWork(work);
    if (chapters.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No readable content found')),
      );
      return;
    }
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => LocalReader(work: work, chapters: chapters),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Local Library'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: _addPath,
            tooltip: 'Add folder',
          ),
        ],
      ),
      body: _works.isEmpty
          ? Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.folder_open, size: 64, color: Colors.grey),
                  const SizedBox(height: 16),
                  const Text('No local manga found'),
                  const SizedBox(height: 8),
                  FilledButton.icon(
                    onPressed: _addPath,
                    icon: const Icon(Icons.add),
                    label: const Text('Add folder'),
                  ),
                ],
              ),
            )
          : ListView.builder(
              itemCount: _works.length,
              itemBuilder: (context, index) {
                final work = _works[index];
                return ListTile(
                  leading: const Icon(Icons.menu_book),
                  title: Text(work.title),
                  subtitle: Text('${chaptersForWork(work).length} chapters'),
                  onTap: () => _openWork(work),
                );
              },
            ),
    );
  }
}
