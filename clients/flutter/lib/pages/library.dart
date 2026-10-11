import 'dart:async';

import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/sync_scheduler.dart';
import '../service/vault_events.dart';
import '../service/vault_service.dart';
import 'continue_reading.dart';
import 'pager_bar.dart';
import 'work_page.dart';

class LibraryPage extends StatefulWidget {
  const LibraryPage({super.key, required this.vault, this.remote});

  final VaultService vault;
  final RemoteService? remote;

  @override
  State<LibraryPage> createState() => _LibraryPageState();
}

class _LibraryPageState extends State<LibraryPage> {
  List<LibraryItem>? items;
  Map<String, ChapterProgressStats> overview = {};
  List<CategoryInfo> categories = [];
  int total = 0;
  int offset = 0;
  String sort = 'added';
  String textQuery = '';
  String kindFilter = 'all';
  String? categoryFilter;
  bool refreshBusy = false;
  String? error;
  Timer? _debounce;

  List<LibraryItem> get visible {
    final result = [...?items];
    if (sort == 'title') {
      result.sort(
        (a, b) =>
            a.work.title.toLowerCase().compareTo(b.work.title.toLowerCase()),
      );
    }
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
    _debounce?.cancel();
    super.dispose();
  }

  void _onEvent(String workId) {
    if (!mounted) return;
    _load();
  }

  Future<void> _load() async {
    try {
      final loaded = await widget.vault.listLibrary(
        offset: offset,
        limit: pageSize,
        filter: LibraryFilter(
          kind: kindFilter == 'all' ? null : kindFilter,
          categoryId: categoryFilter,
          query: textQuery,
        ),
      );
      final counts = await widget.vault.libraryOverview().catchError(
        (_) => <String, ChapterProgressStats>{},
      );
      final cats = await widget.vault.listCategories().catchError(
        (_) => <CategoryInfo>[],
      );
      if (!mounted) return;
      setState(() {
        items = loaded.items;
        total = loaded.total;
        overview = counts;
        categories = cats;
        error = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => error = e.toString());
    }
  }

  void _shiftPage(int next) {
    setState(() {
      offset = next;
      items = null;
    });
    _load();
  }

  void _setKind(String kind) {
    setState(() {
      kindFilter = kind;
      offset = 0;
    });
    _load();
  }

  void _setCategory(String? categoryId) {
    setState(() {
      categoryFilter = categoryId;
      offset = 0;
    });
    _load();
  }

  Future<void> _createCategory() async {
    final typed = await showDialog<String>(
      context: context,
      builder: (_) => const _CategoryDialog(),
    );
    final name = typed?.trim() ?? '';
    if (name.isEmpty) return;
    try {
      await widget.vault.createCategory(name: name);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Could not create category: $e')));
      return;
    }
    await _load();
  }

  Future<void> _deleteCategory(CategoryInfo category) async {
    try {
      await widget.vault.deleteCategory(categoryId: category.id);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Could not delete category: $e')));
      return;
    }
    if (categoryFilter == category.id && mounted) {
      setState(() => categoryFilter = null);
    }
    await _load();
  }

  void _setQuery(String value) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 250), () {
      if (!mounted) return;
      setState(() {
        textQuery = value;
        offset = 0;
      });
      _load();
    });
  }

  Future<void> _open(LibraryItem item) async {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => WorkPage(
          vault: widget.vault,
          remote: widget.remote,
          details: item.work,
        ),
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
      body: error != null
          ? Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      'Failed to load library',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      error!,
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 16),
                    FilledButton(onPressed: _load, child: const Text('Retry')),
                  ],
                ),
              ),
            )
          : shown == null
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
                              onSelected: (_) => _setKind(kind),
                            ),
                          for (final category in categories)
                            InputChip(
                              label: Text(category.name),
                              selected: categoryFilter == category.id,
                              onSelected: (selected) =>
                                  _setCategory(selected ? category.id : null),
                              onDeleted: () => _deleteCategory(category),
                              deleteIcon: const Icon(Icons.close, size: 18),
                            ),
                          ActionChip(
                            avatar: const Icon(Icons.add, size: 18),
                            label: const Text('New'),
                            onPressed: _createCategory,
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
                              onChanged: _setQuery,
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
                          const SizedBox(width: 8),
                          Text(
                            '$total works',
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ],
                      ),
                    ),
                  ),
                  if (visible.isEmpty)
                    SliverFillRemaining(
                      hasScrollBody: false,
                      child: Center(
                        child: Text(
                          total < 1
                              ? 'Your library is empty'
                              : 'No titles match these filters',
                          style: Theme.of(context).textTheme.bodyMedium,
                        ),
                      ),
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
                                    style: TextStyle(
                                      color: Theme.of(context)
                                          .colorScheme
                                          .onSecondary,
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
                  SliverToBoxAdapter(
                    child: PagerBar(
                      offset: offset,
                      limit: pageSize,
                      total: total,
                      onshift: _shiftPage,
                    ),
                  ),
                ],
              ),
            ),
    );
  }
}

class _CategoryDialog extends StatefulWidget {
  const _CategoryDialog();

  @override
  State<_CategoryDialog> createState() => _CategoryDialogState();
}

class _CategoryDialogState extends State<_CategoryDialog> {
  final name = TextEditingController();

  @override
  void dispose() {
    name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('New category'),
      content: TextField(
        controller: name,
        autofocus: true,
        decoration: const InputDecoration(hintText: 'Category name'),
        onSubmitted: (value) => Navigator.of(context).pop(value),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: () => Navigator.of(context).pop(name.text),
          child: const Text('Create'),
        ),
      ],
    );
  }
}
