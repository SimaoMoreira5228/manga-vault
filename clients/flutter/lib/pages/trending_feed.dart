import 'package:flutter/material.dart';

import '../service/vault_service.dart';
import 'list_footer.dart';
import 'work_cover_card.dart';
import 'skeletons.dart';

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
  bool loadingMore = false;
  bool done = false;
  String? pageFailure;
  int page = 1;
  int _epoch = 0;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _pick(String id) async {
    setState(() => sourceId = id);
    await _load();
  }

  Future<void> _load() async {
    final epoch = ++_epoch;
    setState(() {
      page = 1;
      failed = false;
      results = null;
      loadingMore = false;
      done = false;
      pageFailure = null;
    });
    try {
      final trending = await widget.vault.trendingSource(
        sourceId: sourceId,
        page: 1,
      );
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = trending;
        done = trending.isEmpty;
      });
    } catch (_) {
      if (!mounted || epoch != _epoch) return;
      setState(() {
        failed = true;
        results = const [];
        done = true;
      });
    }
  }

  Future<void> _more() async {
    final window = results;
    if (window == null || window.isEmpty) return;
    if (loadingMore || done || failed) return;
    final epoch = _epoch;
    setState(() => loadingMore = true);
    try {
      final trending = await widget.vault.trendingSource(
        sourceId: sourceId,
        page: page + 1,
      );
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = [...window, ...trending];
        page = page + 1;
        pageFailure = null;
        loadingMore = false;
        done = trending.isEmpty;
      });
    } catch (e) {
      if (!mounted || epoch != _epoch) return;
      setState(() {
        pageFailure = '$e';
        loadingMore = false;
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
              ? SkeletonList(
                  label: 'Loading trending from ${source.name}',
                  count: 6,
                  itemBuilder: (context, index) => SkeletonTile(
                    leading: SkeletonBar(height: 28, width: 32, radius: 0),
                    trailing: const SkeletonBar(height: 14, width: 48),
                  ),
                )
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
                          onPressed: _load,
                          child: const Text('Retry'),
                        ),
                      ],
                    ],
                  ),
                )
              : NotificationListener<ScrollNotification>(
                  onNotification: (notification) {
                    if (notification is ScrollUpdateNotification &&
                        notification.metrics.axis == Axis.vertical &&
                        notification.metrics.pixels + 400 >=
                            notification.metrics.maxScrollExtent) {
                      _more();
                    }
                    return false;
                  },
                  child: CustomScrollView(
                    slivers: [
                      SliverPadding(
                        padding: const EdgeInsets.fromLTRB(16, 0, 16, 0),
                        sliver: WorkCoverSliverGrid(
                          itemCount: shown.length,
                          itemBuilder: (context, index) {
                            final entry = shown[index];
                            return WorkCoverCard(
                              vault: widget.vault,
                              title: entry.title,
                              coverUrl: entry.coverUrl,
                              remoteUrl: entry.remoteUrl,
                              kind: source.kind,
                              sourceId: sourceId,
                              badge: '${index + 1}',
                              onTap: () => widget.onOpen(sourceId, entry),
                            );
                          },
                        ),
                      ),
                      SliverToBoxAdapter(
                        child: ListFooter(
                          busy: loadingMore,
                          failure: pageFailure,
                          onRetry: _more,
                        ),
                      ),
                    ],
                  ),
                ),
        ),
      ],
    );
  }
}
