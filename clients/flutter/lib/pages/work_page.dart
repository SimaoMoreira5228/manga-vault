import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/vault_events.dart';
import '../service/sync_scheduler.dart';
import '../service/vault_service.dart';
import 'chapter_actions.dart';
import 'reader_page.dart';
import 'skeletons.dart';

class WorkPage extends StatefulWidget {
  const WorkPage({
    super.key,
    required this.vault,
    required this.details,
    this.remote,
  });

  final VaultService vault;
  final WorkDetails details;
  final RemoteService? remote;

  @override
  State<WorkPage> createState() => _WorkPageState();
}

class _WorkPageState extends State<WorkPage> {
  late Future<WorkDetails> _details;
  Set<String> read = {};
  bool newestFirst = false;
  Set<String> downloaded = {};
  bool inLibrary = false;
  bool refreshing = false;
  bool freshChapters = false;
  List<Map<String, dynamic>> trackLinks = const [];
  Set<String> linkedTrackerIds = {};

  RemoteService? get _account {
    final linked = widget.remote;
    if (linked != null) return linked;
    final vault = widget.vault;
    return vault is RemoteService ? vault : null;
  }

  @override
  void initState() {
    super.initState();
    VaultEvents.instance.subscribe(_onEvent);
    _loadLinkedTrackers();
    _details = _load();
  }

  Future<void> _loadLinkedTrackers() async {
    final account = _account;
    if (account == null || !account.supportsTrackers) return;
    final accounts = await account.myTrackerAccounts().catchError(
      (_) => <Map<String, dynamic>>[],
    );
    if (!mounted) return;
    setState(
      () => linkedTrackerIds = {
        for (final entry in accounts) entry['tracker_id'] as String,
      },
    );
    if (linkedTrackerIds.isNotEmpty) await _loadTracks();
  }

  @override
  void dispose() {
    VaultEvents.instance.unsubscribe(_onEvent);
    super.dispose();
  }

  void _onEvent(String workId) {
    if (workId != widget.details.id || !mounted) return;
    setState(() => freshChapters = true);
  }

  Future<void> _loadTracks() async {
    final account = _account;
    if (account == null) return;
    final links = await account
        .workTracks(workId: widget.details.id)
        .catchError((_) => <Map<String, dynamic>>[]);
    if (mounted) setState(() => trackLinks = links);
  }

  Future<void> _bindTrack(String trackerId, String remoteId) async {
    final account = _account;
    if (account == null) return;
    await account.bindWorkTrack(
      workId: widget.details.id,
      trackerId: trackerId,
      remoteId: remoteId,
    );
    await _loadTracks();
  }

  Future<void> _unbindTrack(String linkId) async {
    final account = _account;
    if (account == null) return;
    await account.deleteWorkTrack(workId: widget.details.id, linkId: linkId);
    await _loadTracks();
  }

  Future<void> _refreshTrack(String linkId) async {
    final account = _account;
    if (account == null) return;
    await account.refreshWorkTrackLink(
      workId: widget.details.id,
      linkId: linkId,
    );
    await _loadTracks();
  }

  Future<void> _showTrackDialog() async {
    final remoteId = TextEditingController();
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Track on ${linkedTrackerIds.first}'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            for (final link in trackLinks)
              ListTile(
                dense: true,
                title: Text('${link['remote_title']} (${link['tracker_id']})'),
                subtitle: Text('ch. ${link['last_chapters_synced'] ?? 0}'),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      icon: const Icon(Icons.refresh, size: 20),
                      onPressed: () => _refreshTrack(link['id'] as String),
                    ),
                    IconButton(
                      icon: const Icon(Icons.link_off, size: 20),
                      onPressed: () => _unbindTrack(link['id'] as String),
                    ),
                  ],
                ),
              ),
            TextField(
              controller: remoteId,
              autofocus: true,
              decoration: const InputDecoration(
                hintText: 'Remote media id (e.g. 30013)',
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
          FilledButton(
            onPressed: () async {
              if (remoteId.text.trim().isEmpty) return;
              try {
                await _bindTrack(linkedTrackerIds.first, remoteId.text.trim());
                if (!context.mounted) return;
                Navigator.of(context).pop();
              } catch (error) {
                if (!context.mounted) return;
                ScaffoldMessenger.of(
                  context,
                ).showSnackBar(SnackBar(content: Text('Bind failed: $error')));
              }
            },
            child: const Text('Bind'),
          ),
        ],
      ),
    );
  }

  Future<WorkDetails> _load() async {
    final fresh = await widget.vault.getWork(workId: widget.details.id);
    freshChapters = false;
    read = (await widget.vault.readChapters(workId: fresh.id)).toSet();
    downloaded =
        (await widget.vault
                .downloadedChapters(workId: fresh.id)
                .catchError((_) => <String>[]))
            .toSet();
    return fresh;
  }

  Future<void> _toggleLibrary(String workId) async {
    if (inLibrary) {
      await widget.vault.removeFromLibrary(workId: workId);
    } else {
      await widget.vault.addToLibrary(workId: workId);
    }
    setState(() => inLibrary = !inLibrary);
  }

  Future<void> _openChapter(List<ChapterSummary> chapters, int index) async {
    final chapter = chapters[index];
    if (!mounted) return;
    await widget.vault.markRead(chapterId: chapter.id);
    if (!mounted) return;
    setState(() => read.add(chapter.id));
    SyncScheduler.instance.nudge();
    Navigator.of(context)
        .push(
          MaterialPageRoute(
            builder: (_) => ReaderPage(
              vault: widget.vault,
              chapters: chapters,
              index: index,
              workId: widget.details.id,
            ),
          ),
        )
        .then((_) => _refresh());
  }

  Future<void> _refreshFromSource() async {
    setState(() => refreshing = true);
    try {
      await widget.vault.refreshWork(workId: widget.details.id);
      await _refresh();
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('Refreshed from source')));
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Refresh failed: $e')));
      }
    } finally {
      if (mounted) setState(() => refreshing = false);
    }
  }

  Future<void> _refresh() async {
    if (!mounted) return;
    final reload = _load();
    setState(() {
      _details = reload;
    });
    await reload;
  }

  Future<void> _showMigrateDialog() async {
    final sources = await widget.vault.listSources();
    final details = await _details;
    if (!mounted) return;
    String? targetSource;
    List<MigrationCandidateInfo> candidates = [];
    String? pickedUrl;
    String? message;

    await showDialog(
      context: context,
      builder: (ctx) {
        return StatefulBuilder(
          builder: (ctx, setDialogState) {
            return AlertDialog(
              title: Text('Migrate "${details.title}"'),
              content: SizedBox(
                width: 380,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    DropdownButtonFormField<String>(
                      decoration: const InputDecoration(
                        labelText: 'Target source',
                      ),
                      items: sources
                          .map(
                            (s) => DropdownMenuItem(
                              value: s.id,
                              child: Text(s.name),
                            ),
                          )
                          .toList(),
                      onChanged: (value) async {
                        if (value == null) return;
                        setDialogState(() {
                          targetSource = value;
                          candidates = [];
                          pickedUrl = null;
                          message = 'Searching…';
                        });
                        try {
                          final found = await widget.vault.migrationCandidates(
                            workId: details.id,
                            toSource: value,
                          );
                          setDialogState(() {
                            candidates = found;
                            pickedUrl = found.firstOrNull?.remoteUrl;
                            message = found.isEmpty ? 'No matches found' : null;
                          });
                        } catch (e) {
                          setDialogState(() => message = 'Search failed: $e');
                        }
                      },
                    ),
                    const SizedBox(height: 12),
                    if (message != null)
                      Text(message!, style: Theme.of(ctx).textTheme.bodySmall),
                    if (candidates.isNotEmpty)
                      SizedBox(
                        height: 200,
                        child: ListView(
                          children: [
                            for (final c in candidates)
                              RadioListTile<String>(
                                value: c.remoteUrl,
                                groupValue: pickedUrl,
                                onChanged: (v) =>
                                    setDialogState(() => pickedUrl = v),
                                title: Text(
                                  c.title,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(ctx).pop(),
                  child: const Text('Cancel'),
                ),
                if (pickedUrl != null)
                  TextButton(
                    onPressed: () async {
                      Navigator.of(ctx).pop();
                      try {
                        final results = await widget.vault.migrationApply(
                          toSource: targetSource!,
                          pairs: [(details.id, pickedUrl!)],
                        );
                        final newId = results.firstOrNull?.toWorkId;
                        if (newId != null && mounted) {
                          final newDetails = await widget.vault.getWork(
                            workId: newId,
                          );
                          if (mounted) {
                            Navigator.of(context).pushReplacement(
                              MaterialPageRoute(
                                builder: (_) => WorkPage(
                                  vault: widget.vault,
                                  remote: widget.remote,
                                  details: newDetails,
                                ),
                              ),
                            );
                          }
                        }
                      } catch (_) {}
                    },
                    child: const Text('Migrate'),
                  ),
              ],
            );
          },
        );
      },
    );
  }

  Future<void> _toggleDownload(String chapterId, bool isDownloaded) async {
    if (isDownloaded) {
      await widget.vault.removeDownload(chapterId: chapterId);
    } else {
      try {
        await widget.vault.downloadChapter(chapterId: chapterId);
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context)
              .showSnackBar(SnackBar(content: Text('Download failed: $e')));
        }
        return;
      }
    }
    setState(() {
      isDownloaded ? downloaded.remove(chapterId) : downloaded.add(chapterId);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: FutureBuilder<WorkDetails>(
        future: _details,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return Semantics(
              container: true,
              label: 'Loading chapters for ${widget.details.title}',
              child: ExcludeSemantics(
                child: CustomScrollView(
                  slivers: [
                    SliverAppBar(title: Text(widget.details.title)),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: const [
                            SkeletonBar(height: 28, width: 220),
                            SizedBox(height: 12),
                            SkeletonBar(height: 14, width: 140),
                          ],
                        ),
                      ),
                    ),
                    SliverList(
                      delegate: SliverChildBuilderDelegate(
                        (context, index) => SkeletonTile(
                          subtitle: index % 3 != 0,
                          trailing: const SkeletonBar(height: 24, width: 24),
                        ),
                        childCount: 10,
                      ),
                    ),
                  ],
                ),
              ),
            );
          }
          final details = snapshot.data ?? widget.details;
          final orderedChapters = newestFirst
              ? details.chapters.reversed.toList()
              : details.chapters;
          return CustomScrollView(
            slivers: [
              SliverAppBar(
                title: Text(details.title),
                actions: [
                  IconButton(
                    icon: refreshing
                        ? const SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.refresh),
                    onPressed: refreshing ? null : _refreshFromSource,
                    tooltip: 'Check for updates',
                  ),
                  IconButton(
                    icon: const Icon(Icons.swap_vert),
                    onPressed: () => setState(() => newestFirst = !newestFirst),
                    tooltip: newestFirst ? 'Oldest first' : 'Newest first',
                  ),
                  IconButton(
                    icon: Icon(
                      inLibrary ? Icons.favorite : Icons.favorite_border,
                    ),
                    onPressed: () => _toggleLibrary(details.id),
                  ),
                  PopupMenuButton<String>(
                    itemBuilder: (_) => [
                      const PopupMenuItem(
                        value: 'migrate',
                        child: Text('Migrate to another source'),
                      ),
                    ],
                    onSelected: (action) {
                      if (action == 'migrate') _showMigrateDialog();
                    },
                  ),
                ],
              ),
              SliverToBoxAdapter(
                child: _Hero(
                  details: details,
                  imageUrl: widget.vault.imageUrl,
                  imageHeaders: widget.vault.imageHeaders,
                ),
              ),
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 8,
                  ),
                  child: Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: [
                      for (final genre in details.genres)
                        Chip(label: Text(genre)),
                    ],
                  ),
                ),
              ),
              const SliverToBoxAdapter(
                child: Padding(
                  padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
                  child: Text(
                    'Chapters',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      letterSpacing: 0.6,
                    ),
                  ),
                ),
              ),
              if (linkedTrackerIds.isNotEmpty)
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.auto_stories_outlined),
                      label: Text(
                        trackLinks.isEmpty
                            ? 'Track on ${linkedTrackerIds.first}'
                            : 'Tracking (${trackLinks.length})',
                      ),
                      onPressed: _showTrackDialog,
                    ),
                  ),
                ),
              if (freshChapters)
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: OutlinedButton.icon(
                      icon: const Icon(Icons.new_releases_outlined),
                      label: const Text('New chapters available'),
                      onPressed: () => _refresh(),
                    ),
                  ),
                ),
              SliverList(
                delegate: SliverChildBuilderDelegate((context, index) {
                  final chapter = orderedChapters[index];
                  final canonicalIndex = details.chapters.indexOf(chapter);
                  final isRead = read.contains(chapter.id);
                  final isDownloaded = downloaded.contains(chapter.id);
                  return ListTile(
                    title: Text(
                      chapter.title,
                      style: TextStyle(
                        color: isRead
                            ? Theme.of(context).colorScheme.secondary
                            : null,
                      ),
                    ),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        if (widget.vault.supportsDownloads)
                          IconButton(
                            icon: Icon(
                              isDownloaded
                                  ? Icons.offline_pin
                                  : Icons.download_outlined,
                              color: isDownloaded
                                  ? Theme.of(context).colorScheme.secondary
                                  : null,
                            ),
                            onPressed: () =>
                                _toggleDownload(chapter.id, isDownloaded),
                          ),
                        if (isRead)
                          const Icon(Icons.check_circle_outline)
                        else
                          const Icon(Icons.chevron_right),
                      ],
                    ),
                    onTap: () => _openChapter(details.chapters, canonicalIndex),
                    onLongPress: () => showChapterActions(
                      context,
                      vault: widget.vault,
                      workId: details.id,
                      chapters: details.chapters,
                      index: canonicalIndex,
                      onChanged: _refresh,
                    ),
                  );
                }, childCount: details.chapters.length),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _Hero extends StatelessWidget {
  const _Hero({
    required this.details,
    required this.imageUrl,
    required this.imageHeaders,
  });

  final WorkDetails details;
  final String Function(String url) imageUrl;
  final Map<String, String> imageHeaders;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final label = TextStyle(
      fontFamily: 'Geist',
      fontSize: 12,
      fontWeight: FontWeight.w500,
      letterSpacing: 0.6,
      color: scheme.onSurfaceVariant,
    );
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(16),
            child: details.coverUrl != null
                ? Image.network(
                    imageUrl(details.coverUrl!),
                    headers: imageHeaders,
                    width: 160,
                    height: 240,
                    fit: BoxFit.cover,
                  )
                : Container(
                    width: 160,
                    height: 240,
                    color: scheme.surfaceContainerHigh,
                    child: const Icon(Icons.menu_book, size: 48),
                  ),
          ),
          const SizedBox(width: 20),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  details.title,
                  style: Theme.of(context).textTheme.displaySmall,
                ),
                if (details.authors.isNotEmpty) ...[
                  const SizedBox(height: 12),
                  Text('AUTHOR', style: label),
                  Text(
                    details.authors.join(', '),
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                ],
                if (details.status != null) ...[
                  const SizedBox(height: 12),
                  Text('STATUS', style: label),
                  Text(
                    details.status!,
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
