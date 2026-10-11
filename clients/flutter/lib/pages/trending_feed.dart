import 'package:flutter/material.dart';

import '../service/vault_service.dart';
import 'source_page_bar.dart';

class TrendingFeed extends StatefulWidget {
  const TrendingFeed({
    super.key,
    required this.vault,
    required this.sources,
    required this.onOpen,
  });

  final VaultService vault;
  final List<SourceSummary> sources;
  final void Function(String sourceId, WorkSummary work) onOpen;

  @override
  State<TrendingFeed> createState() => _TrendingFeedState();
}

class _TrendingFeedState extends State<TrendingFeed> {
  late String sourceId = widget.sources.first.id;
  List<WorkSummary>? results;
  bool failed = false;
  int page = 1;

  @override
  void initState() {
    super.initState();
    _fetch(1);
  }

  Future<void> _pick(String id) async {
    setState(() => sourceId = id);
    await _fetch(1);
  }

  Future<void> _fetch(int target) async {
    setState(() {
      page = target;
      failed = false;
      results = null;
    });
    try {
      final trending = await widget.vault.trendingSource(
        sourceId: sourceId,
        page: target,
      );
      if (!mounted) return;
      setState(() => results = trending);
    } catch (_) {
      if (!mounted) return;
      setState(() {
        failed = true;
        results = const [];
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final shown = results;
    final source = widget.sources.firstWhere(
      (entry) => entry.id == sourceId,
      orElse: () => widget.sources.first,
    );
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
          child: Align(
            alignment: Alignment.centerLeft,
            child: Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                for (final option in widget.sources)
                  ChoiceChip(
                    label: Text(option.name),
                    selected: option.id == sourceId,
                    onSelected: (_) => _pick(option.id),
                  ),
              ],
            ),
          ),
        ),
        Expanded(
          child: shown == null
              ? const Center(child: CircularProgressIndicator())
              : shown.isEmpty
              ? Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        failed
                            ? 'Could not load trending'
                            : 'Nothing trending yet',
                      ),
                      if (failed) ...[
                        const SizedBox(height: 16),
                        FilledButton(
                          onPressed: () => _fetch(page),
                          child: const Text('Retry'),
                        ),
                      ],
                    ],
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.fromLTRB(8, 0, 16, 0),
                  itemCount: shown.length,
                  itemBuilder: (context, index) => ListTile(
                    leading: SizedBox(
                      width: 32,
                      child: Text(
                        '${index + 1}',
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                    ),
                    title: Text(
                      shown[index].title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    subtitle: Text(source.kind),
                    onTap: () => widget.onOpen(sourceId, shown[index]),
                  ),
                ),
        ),
        if (shown != null && shown.isNotEmpty)
          SourcePageBar(page: page, canAdvance: true, onshift: _fetch),
      ],
    );
  }
}
