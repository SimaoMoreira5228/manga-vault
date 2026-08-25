import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import '../service/vault_service.dart';

class DownloadsPage extends StatefulWidget {
  const DownloadsPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<DownloadsPage> createState() => _DownloadsPageState();
}

class _DownloadsPageState extends State<DownloadsPage> {
  List<Map<String, dynamic>> _works = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final items = await widget.vault.listLibrary();
    final downloaded = <Map<String, dynamic>>[];
    for (final item in items) {
      try {
        final chapters = await widget.vault.downloadedChapters(
          workId: item.work.id,
        );
        if (chapters.isNotEmpty) {
          downloaded.add({'work': item.work, 'chapters': chapters});
        }
      } catch (_) {}
    }
    if (!mounted) return;
    setState(() {
      _works = downloaded;
      _loading = false;
    });
  }

  Future<void> _clearWork(String workId) async {
    final chapters = await widget.vault.downloadedChapters(workId: workId);
    for (final chapterId in chapters) {
      await widget.vault.removeDownload(chapterId: chapterId);
    }
    await _load();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Downloads')),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _works.isEmpty
          ? const Center(child: Text('No downloaded chapters'))
          : ListView.builder(
              itemCount: _works.length,
              itemBuilder: (context, index) {
                final work = _works[index]['work'];
                final chapters = _works[index]['chapters'] as List<String>;
                return ExpansionTile(
                  title: Text(work.title),
                  subtitle: Text('${chapters.length} chapters'),
                  children: [
                    for (final chapterId in chapters)
                      ListTile(
                        dense: true,
                        title: Text(chapterId, overflow: TextOverflow.ellipsis),
                        trailing: IconButton(
                          icon: const Icon(Icons.delete_outline, size: 18),
                          onPressed: () async {
                            await widget.vault.removeDownload(
                              chapterId: chapterId,
                            );
                            await _load();
                          },
                        ),
                      ),
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      child: Align(
                        alignment: Alignment.centerRight,
                        child: TextButton(
                          onPressed: () => _clearWork(work.id),
                          child: const Text(
                            'Clear all',
                            style: TextStyle(color: Colors.red),
                          ),
                        ),
                      ),
                    ),
                  ],
                );
              },
            ),
    );
  }
}
