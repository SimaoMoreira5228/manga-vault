import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/vault_service.dart';
import 'list_footer.dart';
import 'skeletons.dart';
import 'trending_feed.dart';
import 'work_cover_card.dart';
import 'work_page.dart';

class DiscoverPage extends StatefulWidget {
  const DiscoverPage({super.key, required this.vault, this.remote});

  final VaultService vault;
  final RemoteService? remote;

  @override
  State<DiscoverPage> createState() => _DiscoverPageState();
}

class _DiscoverPageState extends State<DiscoverPage> {
  List<SourceSummary> sources = [];
  int tabIndex = 0;
  bool loading = true;
  String? error;

  SourceSummary? get _tabSource => tabIndex < 2 ? null : sources[tabIndex - 2];

  @override
  void initState() {
    super.initState();
    _loadSources();
  }

  Future<void> _loadSources() async {
    setState(() {
      loading = true;
      error = null;
    });
    try {
      final loaded = await widget.vault.listSources();
      if (!mounted) return;
      setState(() {
        sources = loaded;
        loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        error = e.toString();
        loading = false;
      });
    }
  }

  Future<void> _open(String sourceId, WorkSummary work) async {
    final details = await widget.vault.importWork(
      sourceId: sourceId,
      remoteUrl: work.remoteUrl,
    );
    if (!mounted) return;
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => WorkPage(
          vault: widget.vault,
          remote: widget.remote,
          details: details,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (loading) {
      return Scaffold(
        appBar: AppBar(title: const Text('Discover')),
        body: const SkeletonGrid(label: 'Loading sources', count: 8),
      );
    }
    if (error != null) {
      return Scaffold(
        appBar: AppBar(title: const Text('Discover')),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'Failed to load sources',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                Text(
                  error!,
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: _loadSources,
                  child: const Text('Retry'),
                ),
              ],
            ),
          ),
        ),
      );
    }
    final tabSource = _tabSource;
    return DefaultTabController(
      length: sources.length + 2,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Discover'),
          bottom: TabBar(
            isScrollable: true,
            onTap: (index) => setState(() => tabIndex = index),
            tabs: [
              const Tab(text: 'Trending'),
              const Tab(text: 'All'),
              for (final source in sources) Tab(text: source.name),
            ],
          ),
        ),
        body: sources.isEmpty
            ? const Center(child: Text('No sources installed'))
            : switch (tabIndex) {
                0 => TrendingFeed(
                  vault: widget.vault,
                  sources: sources,
                  onOpen: _open,
                ),
                1 => AggregatedSearch(
                  vault: widget.vault,
                  sources: sources,
                  onOpen: _open,
                ),
                _ => SourceResults(
                  vault: widget.vault,
                  sourceId: tabSource!.id,
                  onOpen: (work) => _open(tabSource.id, work),
                ),
              },
      ),
    );
  }
}

class AggregatedSearch extends StatefulWidget {
  const AggregatedSearch({
    super.key,
    required this.vault,
    required this.sources,
    required this.onOpen,
  });

  final VaultService vault;
  final List<SourceSummary> sources;
  final void Function(String sourceId, WorkSummary work) onOpen;

  @override
  State<AggregatedSearch> createState() => _AggregatedSearchState();
}

class _AggregatedSearchState extends State<AggregatedSearch> {
  final query = TextEditingController();
  List<({String sourceId, String sourceName, List<WorkSummary> hits})> results =
      [];
  final spent = <String>{};
  bool searching = false;
  bool loadingMore = false;
  bool searched = false;
  String? pageFailure;
  int page = 1;
  int _epoch = 0;

  Future<void> _search() {
    spent.clear();
    return _fetch(1);
  }

  Future<void> _fetch(int target, {bool append = false}) async {
    final text = query.text.trim();
    if (text.isEmpty) return;
    final epoch = append ? _epoch : ++_epoch;
    setState(() {
      page = target;
      if (!append) searching = true;
      loadingMore = append;
      pageFailure = null;
    });
    final live = widget.sources.where((source) => !spent.contains(source.id));
    final failures = <String>[];
    final settled = await Future.wait(
      live.map((source) async {
        try {
          final hits = await widget.vault.searchSource(
            sourceId: source.id,
            query: text,
            page: target,
          );
          if (hits.isEmpty) spent.add(source.id);
          return (sourceId: source.id, sourceName: source.name, hits: hits);
        } catch (e) {
          failures.add('${source.name}: $e');
          return (
            sourceId: source.id,
            sourceName: source.name,
            hits: <WorkSummary>[],
          );
        }
      }),
    );
    if (!mounted || epoch != _epoch) return;
    setState(() {
      if (append) {
        final additions = {
          for (final addition in settled) addition.sourceId: addition.hits,
        };
        results = [
          for (final group in results)
            (
              sourceId: group.sourceId,
              sourceName: group.sourceName,
              hits: [...group.hits, ...?additions[group.sourceId]],
            ),
        ];
      } else {
        results = settled.where((group) => group.hits.isNotEmpty).toList()
          ..sort((a, b) => b.hits.length.compareTo(a.hits.length));
      }
      searching = false;
      loadingMore = false;
      searched = true;
      pageFailure = failures.isEmpty ? null : failures.join('; ');
    });
  }

  void _more() {
    if (spent.length >= widget.sources.length) return;
    _fetch(page + 1, append: true);
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(12),
          child: TextField(
            controller: query,
            onSubmitted: (_) => _search(),
            decoration: InputDecoration(
              hintText: 'Search all sources…',
              suffixIcon: IconButton(
                icon: searching
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.search),
                onPressed: searching ? null : _search,
              ),
            ),
          ),
        ),
        if (results.isEmpty && !searching)
          Expanded(
            child: Center(
              child: Text(
                searched
                    ? (pageFailure == null ? 'No results' : 'Search failed')
                    : 'Type a query to search across all sources',
              ),
            ),
          )
        else
          Expanded(
            child: NotificationListener<ScrollNotification>(
              onNotification: (notification) {
                if (notification is ScrollUpdateNotification &&
                    notification.metrics.axis == Axis.vertical &&
                    notification.metrics.pixels + 400 >=
                        notification.metrics.maxScrollExtent) {
                  _more();
                }
                return false;
              },
              child: ListView.builder(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                itemCount: results.length + 1,
                itemBuilder: (context, groupIndex) {
                  if (groupIndex == results.length) {
                    return ListFooter(
                      busy: loadingMore,
                      failure: pageFailure,
                      onRetry: _more,
                    );
                  }
                  final group = results[groupIndex];
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Padding(
                        padding: const EdgeInsets.only(top: 16, bottom: 8),
                        child: Row(
                          children: [
                            Text(
                              group.sourceName,
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                            const SizedBox(width: 8),
                            Text(
                              '${group.hits.length} hits',
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                          ],
                        ),
                      ),
                      GridView.builder(
                        shrinkWrap: true,
                        physics: const NeverScrollableScrollPhysics(),
                        gridDelegate:
                            const SliverGridDelegateWithMaxCrossAxisExtent(
                              maxCrossAxisExtent: 160,
                              mainAxisExtent: 285,
                              crossAxisSpacing: 12,
                              mainAxisSpacing: 16,
                            ),
                        itemCount: group.hits.length,
                        itemBuilder: (context, index) {
                          final hit = group.hits[index];
                          return WorkCoverCard(
                            vault: widget.vault,
                            title: hit.title,
                            coverUrl: hit.coverUrl,
                            remoteUrl: hit.remoteUrl,
                            sourceId: group.sourceId,
                            onTap: () => widget.onOpen(group.sourceId, hit),
                          );
                        },
                      ),
                    ],
                  );
                },
              ),
            ),
          ),
      ],
    );
  }
}

class SourceResults extends StatefulWidget {
  const SourceResults({
    super.key,
    required this.vault,
    required this.sourceId,
    required this.onOpen,
  });

  final VaultService vault;
  final String sourceId;
  final ValueChanged<WorkSummary> onOpen;

  @override
  State<SourceResults> createState() => _SourceResultsState();
}

class _SourceResultsState extends State<SourceResults> {
  final query = TextEditingController();
  List<WorkSummary>? results;
  bool searched = false;
  bool loadingMore = false;
  bool done = false;
  String? pageFailure;
  int page = 1;
  int _epoch = 0;

  @override
  void didUpdateWidget(SourceResults oldWidget) {
    super.didUpdateWidget(oldWidget);
    _loadLatest();
  }

  @override
  void initState() {
    super.initState();
    _loadLatest();
  }

  void _reset() {
    page = 1;
    searched = false;
    results = null;
    loadingMore = false;
    done = false;
    pageFailure = null;
  }

  Future<void> _loadLatest() async {
    _reset();
    final epoch = ++_epoch;
    try {
      final latest = await widget.vault.latestSource(
        sourceId: widget.sourceId,
        page: 1,
      );
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = latest;
        done = latest.isEmpty;
      });
    } catch (_) {
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = const [];
        done = true;
      });
    }
  }

  Future<void> _search() async {
    _reset();
    final epoch = ++_epoch;
    setState(() => searched = true);
    try {
      final found = await widget.vault.searchSource(
        sourceId: widget.sourceId,
        query: query.text,
        page: 1,
      );
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = found;
        done = found.isEmpty;
      });
    } catch (_) {
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = const [];
        done = true;
      });
    }
  }

  Future<void> _more() async {
    final window = results;
    if (window == null || window.isEmpty) return;
    if (loadingMore || done) return;
    final text = query.text.trim();
    final epoch = _epoch;
    setState(() => loadingMore = true);
    try {
      final found = text.isEmpty
          ? await widget.vault.latestSource(
              sourceId: widget.sourceId,
              page: page + 1,
            )
          : await widget.vault.searchSource(
              sourceId: widget.sourceId,
              query: text,
              page: page + 1,
            );
      if (!mounted || epoch != _epoch) return;
      setState(() {
        results = [...window, ...found];
        page = page + 1;
        pageFailure = null;
        loadingMore = false;
        done = found.isEmpty;
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
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(12),
          child: TextField(
            controller: query,
            onSubmitted: (_) => _search(),
            decoration: InputDecoration(
              hintText: 'Search',
              suffixIcon: IconButton(
                icon: const Icon(Icons.search),
                onPressed: _search,
              ),
            ),
          ),
        ),
        Expanded(
          child: shown == null
              ? SkeletonGrid(
                  label: 'Loading the latest releases from ${widget.sourceId}',
                  count: 8,
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
                  child: GridView.builder(
                    padding: const EdgeInsets.all(16),
                    gridDelegate:
                        const SliverGridDelegateWithMaxCrossAxisExtent(
                          maxCrossAxisExtent: 160,
                          mainAxisExtent: 285,
                          crossAxisSpacing: 12,
                          mainAxisSpacing: 16,
                        ),
                    itemCount: shown.length + 1,
                    itemBuilder: (context, index) {
                      if (index == shown.length) {
                        return ListFooter(
                          busy: loadingMore,
                          failure: pageFailure,
                          onRetry: _more,
                        );
                      }
                      final hit = shown[index];
                      return WorkCoverCard(
                        vault: widget.vault,
                        title: hit.title,
                        coverUrl: hit.coverUrl,
                        remoteUrl: hit.remoteUrl,
                        sourceId: widget.sourceId,
                        onTap: () => widget.onOpen(hit),
                      );
                    },
                  ),
                ),
        ),
      ],
    );
  }
}
