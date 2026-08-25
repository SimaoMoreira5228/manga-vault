import 'package:flutter/material.dart';

import '../service/vault_service.dart';
import 'work_page.dart';

class DiscoverPage extends StatefulWidget {
  const DiscoverPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<DiscoverPage> createState() => _DiscoverPageState();
}

class _DiscoverPageState extends State<DiscoverPage> {
  List<SourceSummary> sources = [];
  int tabIndex = 0;

  String? get selectedSource => tabIndex == 0 ? null : sources[tabIndex - 1].id;

  @override
  void initState() {
    super.initState();
    _loadSources();
  }

  Future<void> _loadSources() async {
    sources = await widget.vault.listSources();
    if (mounted) setState(() {});
  }

  Future<void> _open(String sourceId, WorkSummary work) async {
    final details = await widget.vault.importWork(
      sourceId: sourceId,
      remoteUrl: work.remoteUrl,
    );
    if (!mounted) return;
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => WorkPage(vault: widget.vault, details: details),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: sources.length + 1,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Discover'),
          bottom: TabBar(
            isScrollable: true,
            onTap: (index) => setState(() => tabIndex = index),
            tabs: [
              const Tab(text: 'All'),
              for (final source in sources) Tab(text: source.name),
            ],
          ),
        ),
        body: sources.isEmpty
            ? const Center(child: Text('No sources installed'))
            : tabIndex == 0
            ? AggregatedSearch(
                vault: widget.vault,
                sources: sources,
                onOpen: _open,
              )
            : SourceResults(
                vault: widget.vault,
                sourceId: sources[tabIndex - 1].id,
                onOpen: (work) => _open(sources[tabIndex - 1].id, work),
              ),
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
  bool searching = false;

  Future<void> _search() async {
    final text = query.text.trim();
    if (text.isEmpty) return;
    searching = true;
    setState(() {});
    final settled = await Future.wait(
      widget.sources.map((source) async {
        try {
          final hits = await widget.vault.searchSource(
            sourceId: source.id,
            query: text,
            page: 1,
          );
          return (sourceId: source.id, sourceName: source.name, hits: hits);
        } catch (_) {
          return (
            sourceId: source.id,
            sourceName: source.name,
            hits: <WorkSummary>[],
          );
        }
      }),
    );
    if (!mounted) return;
    setState(() {
      results = settled.where((group) => group.hits.isNotEmpty).toList()
        ..sort((a, b) => b.hits.length.compareTo(a.hits.length));
      searching = false;
    });
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
          const Expanded(
            child: Center(
              child: Text('Type a query to search across all sources'),
            ),
          )
        else
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
              itemCount: results.length,
              itemBuilder: (context, groupIndex) {
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
                            childAspectRatio: 2 / 3,
                            crossAxisSpacing: 12,
                            mainAxisSpacing: 12,
                          ),
                      itemCount: group.hits.length,
                      itemBuilder: (context, index) => WorkCard(
                        work: group.hits[index],
                        onOpen: () =>
                            widget.onOpen(group.sourceId, group.hits[index]),
                      ),
                    ),
                  ],
                );
              },
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

  @override
  void didUpdateWidget(SourceResults oldWidget) {
    super.didUpdateWidget(oldWidget);
    results = null;
    _loadLatest();
  }

  @override
  void initState() {
    super.initState();
    _loadLatest();
  }

  Future<void> _loadLatest() async {
    try {
      final latest = await widget.vault.latestSource(
        sourceId: widget.sourceId,
        page: 1,
      );
      if (!mounted) return;
      setState(() => results = latest);
    } catch (_) {
      setState(() => results = []);
    }
  }

  Future<void> _search() async {
    final found = await widget.vault.searchSource(
      sourceId: widget.sourceId,
      query: query.text,
      page: 1,
    );
    setState(() => results = found);
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
              ? const Center(child: CircularProgressIndicator())
              : GridView.builder(
                  padding: const EdgeInsets.all(16),
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 160,
                    childAspectRatio: 2 / 3,
                    crossAxisSpacing: 12,
                    mainAxisSpacing: 12,
                  ),
                  itemCount: shown.length,
                  itemBuilder: (context, index) => WorkCard(
                    work: shown[index],
                    onOpen: () => widget.onOpen(shown[index]),
                  ),
                ),
        ),
      ],
    );
  }
}

class WorkCard extends StatelessWidget {
  const WorkCard({super.key, required this.work, required this.onOpen});

  final WorkSummary work;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onOpen,
      borderRadius: BorderRadius.circular(8),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: Stack(
          fit: StackFit.expand,
          children: [
            work.coverUrl != null
                ? Image.network(work.coverUrl!, fit: BoxFit.cover)
                : const ColoredBox(
                    color: Color(0xFF322820),
                    child: Icon(Icons.menu_book),
                  ),
            Positioned(
              left: 0,
              right: 0,
              bottom: 0,
              child: DecoratedBox(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topCenter,
                    end: Alignment.bottomCenter,
                    colors: [
                      Colors.transparent,
                      Colors.black.withValues(alpha: 0.85),
                    ],
                  ),
                ),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(8, 20, 8, 8),
                  child: Text(
                    work.title,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.titleMedium
                        ?.copyWith(color: Colors.white),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
