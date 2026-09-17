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
  String? error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      entries = null;
      error = null;
    });
    try {
      final result = await widget.vault.history(limit: 100);
      if (!mounted) return;
      setState(() => entries = result);
    } catch (e) {
      if (!mounted) return;
      setState(() => error = e.toString());
    }
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
      body: error != null
          ? _ErrorBody(message: error!, onRetry: _load)
          : shown == null
          ? const Center(child: CircularProgressIndicator())
          : shown.isEmpty
          ? const Center(child: Text('Nothing read yet'))
          : ListView.builder(
              itemCount: shown.length,
              itemBuilder: (context, index) {
                final entry = shown[index];
                final readAt = entry.readAt;
                final previous = index == 0 ? null : shown[index - 1].readAt;
                final showDay =
                    readAt != null &&
                    (previous == null ||
                        _dayLabel(previous) != _dayLabel(readAt));
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (showDay)
                      Padding(
                        padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
                        child: Text(
                          _dayLabel(readAt),
                          style: Theme.of(context).textTheme.titleSmall,
                        ),
                      ),
                    ListTile(
                      title: Text(
                        entry.workTitle,
                        overflow: TextOverflow.ellipsis,
                      ),
                      subtitle: Text(
                        entry.chapterTitle ?? entry.chapterId,
                        overflow: TextOverflow.ellipsis,
                      ),
                      leading: const Icon(Icons.history),
                      trailing: readAt == null
                          ? null
                          : Text(
                              '${readAt.hour.toString().padLeft(2, '0')}:${readAt.minute.toString().padLeft(2, '0')}',
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

class _ErrorBody extends StatelessWidget {
  const _ErrorBody({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'Failed to load history',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              message,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            FilledButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}
