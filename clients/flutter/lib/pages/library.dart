import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/sync_scheduler.dart';
import '../service/vault_events.dart';
import '../service/vault_service.dart';
import 'continue_reading.dart';
import 'work_page.dart';

class LibraryPage extends StatefulWidget {
  const LibraryPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<LibraryPage> createState() => _LibraryPageState();
}

class _LibraryPageState extends State<LibraryPage> {
  List<LibraryItem>? items;
  Map<String, ChapterProgressStats> overview = {};
  List<CategoryInfo> categories = [];
  String sort = 'updated';
  String textQuery = '';
  String kindFilter = 'all';
  String? categoryFilter;
  bool refreshBusy = false;

  bool get isRemote => widget.vault is RemoteService;

  List<LibraryItem> get filtered {
    var result = items ?? [];
    if (kindFilter == 'manga') {
      result = result.where((item) => item.work.kind == 'Manga').toList();
    } else if (kindFilter == 'novel') {
      result = result.where((item) => item.work.kind == 'Novel').toList();
    }
    if (categoryFilter != null) {
      result = result.where((item) => item.entryId == categoryFilter).toList();
    }
    final text = textQuery.toLowerCase().trim();
    if (text.isNotEmpty) {
      result = result
          .where((item) => item.work.title.toLowerCase().contains(text))
          .toList();
    }
    result.sort((a, b) {
      if (sort == 'title') return a.work.title.compareTo(b.work.title);
      return b.entryId.compareTo(a.entryId);
    });
    return result;
  }

  int? unreadCount(String workId) {
    final stats = overview[workId];
    if (stats == null || stats.total == 0) return null;
    final unread = stats.total - stats.read;
    return unread > 0 ? unread : null;
  }

  @override
  void initState() {
    super.initState();
    VaultEvents.instance.subscribe(_onEvent);
    _load();
  }

  @override
  void dispose() {
    VaultEvents.instance.unsubscribe(_onEvent);
    super.dispose();
  }

  void _onEvent(String workId) {
    if (!mounted) return;
    _load();
  }

  Future<void> _load() async {
    final loaded = await widget.vault.listLibrary();
    final counts = await widget.vault.libraryOverview().catchError(
      (_) => <String, ChapterProgressStats>{},
    );
    final cats = isRemote
        ? await widget.vault.listCategories().catchError(
            (_) => <CategoryInfo>[],
          )
        : <CategoryInfo>[];
    if (!mounted) return;
    setState(() {
      items = loaded;
      overview = counts;
      categories = cats;
    });
  }

  Future<void> _open(LibraryItem item) async {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => WorkPage(vault: widget.vault, details: item.work),
      ),
    );
    await _load();
  }

  Future<void> _refreshAll() async {
    refreshBusy = true;
    try {
      final queued = await widget.vault.refreshAllLibrary();
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Queued $queued works for update')),
      );
    } finally {
      if (mounted) setState(() => refreshBusy = false);
    }
  }

  Future<void> _assignCategory(LibraryItem item) async {
    if (!isRemote) return;
    final picked = await showModalBottomSheet<String?>(
      context: context,
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Set category',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
            ),
            ListTile(
              title: const Text('No category'),
              onTap: () => Navigator.of(ctx).pop(null),
            ),
            ...categories.map(
              (cat) => ListTile(
                title: Text(cat.name),
                onTap: () => Navigator.of(ctx).pop(cat.id),
              ),
            ),
            const SizedBox(height: 8),
          ],
        ),
      ),
    );
    if (picked == null && !mounted) return;
    if (!isRemote) return;
    try {
      await widget.vault.setEntryCategory(
        entryId: item.entryId,
        categoryId: picked,
      );
      await _load();
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final shown = items;
    final visible = filtered;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Library'),
        actions: [
          if (refreshBusy)
            const Padding(
              padding: EdgeInsets.all(16),
              child: SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            )
          else
            IconButton(icon: const Icon(Icons.sync), onPressed: _refreshAll),
        ],
      ),
      body: shown == null
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: () async {
                await _load();
                SyncScheduler.instance.nudge();
              },
              child: CustomScrollView(
                physics: const AlwaysScrollableScrollPhysics(),
                slivers: [
                  SliverToBoxAdapter(
                    child: ContinueReadingRow(
                      vault: widget.vault,
                      onChanged: _load,
                    ),
                  ),
                  SliverToBoxAdapter(
                    child: Padding(
                      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                      child: Wrap(
                        spacing: 6,
                        runSpacing: 4,
                        children: [
                          for (final kind in ['all', 'manga', 'novel'])
                            ChoiceChip(
                              label: Text(kind),
                              selected: kindFilter == kind,
                              onSelected: (_) =>
                                  setState(() => kindFilter = kind),
                            ),
                          const SizedBox(width: 8),
                          ChoiceChip(
                            label: const Text('All categories'),
                            selected: categoryFilter == null,
                            onSelected: (_) =>
                                setState(() => categoryFilter = null),
                          ),
                          for (final cat in categories)
                            ChoiceChip(
                              label: Text(cat.name),
                              selected: categoryFilter == cat.id,
                              onSelected: (_) => setState(
                                () => categoryFilter = categoryFilter == cat.id
                                    ? null
                                    : cat.id,
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
                  SliverToBoxAdapter(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 4,
                      ),
                      child: Row(
                        children: [
                          Expanded(
                            child: TextField(
                              decoration: const InputDecoration(
                                isDense: true,
                                hintText: 'Filter titles…',
                                border: OutlineInputBorder(),
                              ),
                              onChanged: (value) =>
                                  setState(() => textQuery = value),
                            ),
                          ),
                          const SizedBox(width: 8),
                          SegmentedButton<String>(
                            selected: {sort},
                            onSelectionChanged: (value) =>
                                setState(() => sort = value.first),
                            segments: const [
                              ButtonSegment(
                                value: 'added',
                                label: Text('Recent'),
                              ),
                              ButtonSegment(value: 'title', label: Text('A-Z')),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                  if (visible.isEmpty)
                    const SliverFillRemaining(
                      hasScrollBody: false,
                      child: Center(child: Text('No matching library entries')),
                    )
                  else
                    SliverList(
                      delegate: SliverChildBuilderDelegate((context, index) {
                        final item = visible[index];
                        final unread = unreadCount(item.work.id);
                        return ListTile(
                          leading: const Icon(Icons.menu_book),
                          title: Row(
                            children: [
                              Expanded(
                                child: Text(
                                  item.work.title,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ),
                              if (unread != null)
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 8,
                                    vertical: 2,
                                  ),
                                  decoration: BoxDecoration(
                                    color: Theme.of(context)
                                        .colorScheme
                                        .secondary,
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                  child: Text(
                                    '$unread new',
                                    style: const TextStyle(
                                      color: Colors.white,
                                      fontSize: 11,
                                    ),
                                  ),
                                ),
                            ],
                          ),
                          subtitle: Text(
                            '${item.work.chapters.length} chapters',
                          ),
                          trailing: PopupMenuButton<String>(
                            onSelected: (action) {
                              if (action == 'category') _assignCategory(item);
                            },
                            itemBuilder: (_) => [
                              const PopupMenuItem(
                                value: 'category',
                                child: Text('Set category'),
                              ),
                            ],
                          ),
                          onTap: () => _open(item),
                        );
                      }, childCount: visible.length),
                    ),
                ],
              ),
            ),
    );
  }
}
