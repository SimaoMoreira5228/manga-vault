import 'package:flutter/material.dart';

import '../service/vault_service.dart';

class HistoryPage extends StatefulWidget {
  const HistoryPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<HistoryPage> createState() => _HistoryPageState();
}

class _HistoryPageState extends State<HistoryPage> {
  List<HistoryEntry>? entries;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final result = await widget.vault.history(limit: 100);
    if (!mounted) return;
    setState(() => entries = result);
  }

  String _dayLabel(DateTime date) {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final yesterday = today.subtract(const Duration(days: 1));
    final d = DateTime(date.year, date.month, date.day);
    if (d == today) return 'Today';
    if (d == yesterday) return 'Yesterday';
    return '${date.day}/${date.month}/${date.year}';
  }

  @override
  Widget build(BuildContext context) {
    final shown = entries;
    return Scaffold(
      appBar: AppBar(title: const Text('History')),
      body: shown == null
          ? const Center(child: CircularProgressIndicator())
          : shown.isEmpty
          ? const Center(child: Text('Nothing read yet'))
          : ListView.builder(
              itemCount: shown.length,
              itemBuilder: (context, index) {
                final entry = shown[index];
                final showDay =
                    index == 0 ||
                    _dayLabel(shown[index - 1].readAt) !=
                        _dayLabel(entry.readAt);
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (showDay)
                      Padding(
                        padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
                        child: Text(
                          _dayLabel(entry.readAt),
                          style: Theme.of(context).textTheme.titleSmall,
                        ),
                      ),
                    ListTile(
                      title: Text(
                        entry.workTitle,
                        overflow: TextOverflow.ellipsis,
                      ),
                      subtitle: Text(
                        entry.chapterTitle,
                        overflow: TextOverflow.ellipsis,
                      ),
                      leading: const Icon(Icons.history),
                      trailing: Text(
                        '${entry.readAt.hour.toString().padLeft(2, '0')}:${entry.readAt.minute.toString().padLeft(2, '0')}',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ),
                  ],
                );
              },
            ),
    );
  }
}
