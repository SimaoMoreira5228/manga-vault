import '../service/vault_events.dart';
import '../service/sync_scheduler.dart';

import 'dart:io';

import 'package:flutter/material.dart';

import '../service/app_prefs.dart';
import '../service/vault_service.dart';

class WorkPage extends StatefulWidget {
  const WorkPage({super.key, required this.vault, required this.details});

  final VaultService vault;
  final WorkDetails details;

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

  @override
  void initState() {
    super.initState();
    VaultEvents.instance.subscribe(_onEvent);
    widget.vault
        .myTrackerAccounts()
        .then((accounts) {
          if (!mounted) return;
          setState(
            () => linkedTrackerIds = accounts
                .map((a) => a['tracker_id'] as String)
                .toSet(),
          );
          if (linkedTrackerIds.isNotEmpty) _loadTracks();
        })
        .catchError((_) {});
    _details = _load();
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
    final links = await widget.vault
        .workTracks(workId: widget.details.id)
        .catchError((_) => <Map<String, dynamic>>[]);
    if (mounted) setState(() => trackLinks = links);
  }

  Future<void> _bindTrack(String trackerId, String remoteId) async {
    await widget.vault.bindWorkTrack(
      workId: widget.details.id,
      trackerId: trackerId,
      remoteId: remoteId,
    );
    await _loadTracks();
  }

  Future<void> _unbindTrack(String linkId) async {
    await widget.vault.deleteWorkTrack(
      workId: widget.details.id,
      linkId: linkId,
    );
    await _loadTracks();
  }

  Future<void> _refreshTrack(String linkId) async {
    await widget.vault.refreshWorkTrackLink(
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

  Future<void> _markPreviousRead(
    List<ChapterSummary> chapters,
    int index,
  ) async {
    final workId = widget.details.id;
    final pending = [
      for (var i = 0; i <= index; i++)
        if (!read.contains(chapters[i].id)) chapters[i].id,
    ];
    if (pending.isEmpty) return;
    await widget.vault.markChapters(
      workId: workId,
      chapterIds: pending,
      read: true,
    );
    if (!mounted) return;
    setState(() => read.addAll(pending));
    SyncScheduler.instance.nudge();
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
    final reload = _load();
    setState(() {
      _details = reload;
    });
    await reload;
  }

  Future<void> _showMigrateDialog() async {
    final sources = await widget.vault.listSources();
    final details = await _details;
    if (!mounted || details == null) return;
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
            return const Center(child: CircularProgressIndicator());
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
              SliverToBoxAdapter(child: _Hero(details: details)),
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
                    onLongPress: () => showModalBottomSheet<void>(
                      context: context,
                      builder: (sheetContext) => SafeArea(
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            ListTile(
                              leading: const Icon(Icons.done_all),
                              title: Text(
                                'Mark previous ${canonicalIndex + 1} as read',
                              ),
                              onTap: () {
                                Navigator.of(sheetContext).pop();
                                _markPreviousRead(
                                  details.chapters,
                                  canonicalIndex,
                                );
                              },
                            ),
                          ],
                        ),
                      ),
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
  const _Hero({required this.details});

  final WorkDetails details;

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
                    details.coverUrl!,
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

class ReaderPage extends StatefulWidget {
  const ReaderPage({
    super.key,
    required this.vault,
    required this.chapters,
    required this.index,
    required this.workId,
  });

  final VaultService vault;
  final List<ChapterSummary> chapters;
  final int index;
  final String workId;

  @override
  State<ReaderPage> createState() => _ReaderPageState();
}

class _ReaderPageState extends State<ReaderPage> {
  late ChapterSummary current;
  late ChapterBody? body;
  final ScrollController _scroll = ScrollController();
  double? _resumeFraction;
  int _nextPreloaded = -1;
  int _lastSaveMs = 0;
  double _fontSize = 16;
  double _lineHeight = 1.6;
  bool _pagedMode = false;
  double _imageMargin = 0;
  double _imageGap = 0;
  bool _rtlMode = false;
  String _workId = '';

  ChapterSummary? get _nextChapter {
    final next = widget.index - 1;
    return next >= 0 && next < widget.chapters.length
        ? widget.chapters[next]
        : null;
  }

  void _onScroll() {
    if (!_scroll.hasClients) return;
    final maxExtent = _scroll.position.maxScrollExtent;
    if (maxExtent <= 0) return;
    final fraction = (_scroll.offset / maxExtent).clamp(0.0, 1.0);

    if (fraction > 0.8) {
      final next = _nextChapter;
      if (next != null && _nextPreloaded != 1) {
        _nextPreloaded = 1;
        widget.vault.chapterContent(chapterId: next.id);
      }
    }

    final now = DateTime.now().millisecondsSinceEpoch;
    if (now - _lastSaveMs > 800) {
      _lastSaveMs = now;
      AppPrefs.instance().then(
        (prefs) => prefs.setPosition(current.id, fraction),
      );
    }
  }

  String? translatedHtml;
  List<Map<String, dynamic>> matches = const [];
  String? translationMode;

  @override
  void initState() {
    super.initState();
    current = widget.chapters[widget.index];
    _workId = widget.workId;
    body = null;
    _scroll.addListener(_onScroll);
    _loadSettings();
    _load(current.id);
    widget.vault
        .translationMode()
        .then((mode) {
          if (mounted) setState(() => translationMode = mode);
        })
        .catchError((_) {});
  }

  Future<void> _loadSettings() async {
    final prefs = await AppPrefs.instance();
    if (!mounted) return;
    setState(() {
      _fontSize = prefs.effectiveFontSize(_workId);
      _lineHeight = prefs.effectiveLineHeight(_workId);
      _pagedMode = prefs.effectivePagedMode(_workId);
      _imageMargin = prefs.effectiveImageMargin(_workId);
      _imageGap = prefs.effectiveImageGap(_workId);
      _rtlMode = prefs.effectiveRtlMode(_workId);
    });
  }

  bool get _canTranslate =>
      translatedHtml == null &&
      translationMode != null &&
      translationMode != 'off' &&
      translationMode != 'unavailable';

  Future<void> _translate() async {
    final target = TextEditingController(text: 'en');
    final source = TextEditingController();
    final confirmed = await showDialog<List<String>>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Translate'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: target,
              decoration: const InputDecoration(hintText: 'To (e.g. en)'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: source,
              decoration: const InputDecoration(
                hintText: 'From (optional, enables glossary)',
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () =>
                Navigator.of(context)
                    .pop([target.text.trim(), source.text.trim()]),
            child: const Text('Translate'),
          ),
        ],
      ),
    );
    if (confirmed == null || confirmed.first.isEmpty) return;
    final result = await widget.vault.translateChapter(
      chapterId: current.id,
      to: confirmed.first,
      from: confirmed.last.isEmpty ? null : confirmed.last,
    );
    if (!mounted) return;
    setState(() {
      translatedHtml = result['content'] as String?;
      matches =
          (result['matches'] as List?)?.cast<Map<String, dynamic>>() ??
          const [];
    });
  }

  void _handleReaderSetting(String action) async {
    final prefs = await AppPrefs.instance();
    switch (action) {
      case 'font_up':
        _fontSize = (_fontSize + 2).clamp(10, 32);
        await prefs.setWorkReaderSetting(_workId, 'fontSize', _fontSize);
      case 'font_down':
        _fontSize = (_fontSize - 2).clamp(10, 32);
        await prefs.setWorkReaderSetting(_workId, 'fontSize', _fontSize);
      case 'spacing':
        _lineHeight = _lineHeight == 1.6 ? 2.2 : 1.6;
        await prefs.setWorkReaderSetting(_workId, 'lineHeight', _lineHeight);
      case 'paged':
        _pagedMode = !_pagedMode;
        await prefs.setWorkReaderSetting(_workId, 'pagedMode', _pagedMode);
      case 'margin_up':
        _imageMargin = (_imageMargin + 4).clamp(0, 64);
        await prefs.setWorkReaderSetting(_workId, 'imageMargin', _imageMargin);
      case 'margin_down':
        _imageMargin = (_imageMargin - 4).clamp(0, 64);
        await prefs.setWorkReaderSetting(_workId, 'imageMargin', _imageMargin);
      case 'gap_up':
        _imageGap = (_imageGap + 2).clamp(0, 32);
        await prefs.setWorkReaderSetting(_workId, 'imageGap', _imageGap);
      case 'gap_down':
        _imageGap = (_imageGap - 2).clamp(0, 32);
        await prefs.setWorkReaderSetting(_workId, 'imageGap', _imageGap);
      case 'rtl':
        _rtlMode = !_rtlMode;
        await prefs.setWorkReaderSetting(_workId, 'rtlMode', _rtlMode);
    }
    setState(() {});
  }

  void _showGlossary() {
    showModalBottomSheet<void>(
      context: context,
      builder: (context) => SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            for (final entry in matches) ...[
              Text(
                entry['term'] as String,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              if (entry['romanization'] != null)
                Text(
                  entry['romanization'] as String,
                  style: Theme.of(context).textTheme.bodySmall
                      ?.copyWith(fontStyle: FontStyle.italic),
                ),
              for (final meaning
                  in (entry['meanings'] as List).cast<Map<String, dynamic>>())
                ListTile(
                  dense: true,
                  title: Text(meaning['meaning'] as String),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text('${meaning['votes']}'),
                      IconButton(
                        icon: Icon(
                          meaning['voted_by_me'] == true
                              ? Icons.thumb_up
                              : Icons.thumb_up_outlined,
                          size: 18,
                        ),
                        onPressed: () async {
                          final voted = await widget.vault.toggleGlossaryVote(
                            meaningId: meaning['id'] as String,
                          );
                          setState(() => meaning['voted_by_me'] = voted);
                          setState(
                            () => meaning['votes'] =
                                (meaning['votes'] as int) + (voted ? 1 : -1),
                          );
                        },
                      ),
                    ],
                  ),
                ),
              const Divider(),
            ],
          ],
        ),
      ),
    );
  }

  Future<void> _load(String chapterId) async {
    final cached = cachedChapterContent(chapterId);
    final content =
        cached ?? await widget.vault.chapterContent(chapterId: chapterId);
    await widget.vault.markRead(chapterId: chapterId);
    if (!mounted) return;
    setState(() {
      body = content;
      translatedHtml = null;
    });
    _nextPreloaded = cached != null ? 1 : -1;

    final prefs = await AppPrefs.instance();
    final saved = prefs.positionFor(chapterId);
    if (saved != null && saved > 0.03 && saved < 0.97) {
      _resumeFraction = saved;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted || !_scroll.hasClients || _resumeFraction == null) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Resume at ${(_resumeFraction! * 100).round()}%'),
            action: SnackBarAction(
              label: 'Resume',
              onPressed: () {
                final maxExtent = _scroll.position.maxScrollExtent;
                if (maxExtent > 0) _scroll.jumpTo(maxExtent * _resumeFraction!);
              },
            ),
          ),
        );
      });
    } else {
      _resumeFraction = null;
    }
  }

  Future<void> _go(int delta) async {
    final next = widget.index + delta;
    if (next < 0 || next >= widget.chapters.length) return;
    setState(() {
      current = widget.chapters[next];
      body = null;
    });
    await _load(current.id);
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text(current.title, overflow: TextOverflow.ellipsis),
            Text(
              '${widget.index + 1} / ${widget.chapters.length}',
              style: TextStyle(fontSize: 11, color: scheme.onSurfaceVariant),
            ),
          ],
        ),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Navigator.of(context).pop(),
        ),
        actions: [
          if (matches.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.menu_book_outlined),
              onPressed: _showGlossary,
            ),
          if (_canTranslate)
            IconButton(
              icon: const Icon(Icons.translate),
              onPressed: _translate,
            ),
          PopupMenuButton<String>(
            icon: const Icon(Icons.tune),
            onSelected: _handleReaderSetting,
            itemBuilder: (_) => [
              PopupMenuItem(value: 'font_up', child: Text('Larger text')),
              PopupMenuItem(value: 'font_down', child: Text('Smaller text')),
              PopupMenuItem(
                value: 'spacing',
                child: Text('Toggle line spacing'),
              ),
              PopupMenuItem(value: 'paged', child: Text('Toggle paged mode')),
              PopupMenuDivider(),
              PopupMenuItem(
                value: 'margin_up',
                child: Text('Wider image margins'),
              ),
              PopupMenuItem(
                value: 'margin_down',
                child: Text('Narrower image margins'),
              ),
              PopupMenuItem(
                value: 'gap_up',
                child: Text('More gap between images'),
              ),
              PopupMenuItem(
                value: 'gap_down',
                child: Text('Less gap between images'),
              ),
              PopupMenuDivider(),
              PopupMenuItem(
                value: 'rtl',
                child: Text(_rtlMode ? 'Switch to LTR' : 'Switch to RTL'),
              ),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.skip_previous),
            onPressed: widget.index > 0 ? () => _go(-1) : null,
          ),
          IconButton(
            icon: const Icon(Icons.skip_next),
            onPressed: widget.index + 1 < widget.chapters.length
                ? () => _go(1)
                : null,
          ),
        ],
      ),
      body: Directionality(
        textDirection: _rtlMode ? TextDirection.rtl : TextDirection.ltr,
        child: switch (body) {
          null => const Center(child: CircularProgressIndicator()),
          ChapterBody_Images urls =>
            _pagedMode
                ? _pagedImageBody(urls)
                : InteractiveViewer(
                    child: ListView.builder(
                      controller: _scroll,
                      padding: EdgeInsets.symmetric(horizontal: _imageMargin),
                      itemExtent: null,
                      itemBuilder: (context, index) {
                        final page = urls.field0[index];
                        final image = page.startsWith('file://')
                            ? Image.file(
                                File.fromUri(Uri.parse(page)),
                                fit: BoxFit.fitWidth,
                              )
                            : Image.network(page, fit: BoxFit.fitWidth);
                        return index > 0
                            ? Padding(
                                padding: EdgeInsets.only(top: _imageGap),
                                child: image,
                              )
                            : image;
                      },
                    ),
                  ),
          ChapterBody_Html html => Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 720),
              child: SingleChildScrollView(
                controller: _scroll,
                padding: const EdgeInsets.all(24),
                child: Text(
                  _stripTags(translatedHtml ?? html.field0),
                  style: Theme.of(context).textTheme.bodyLarge
                      ?.copyWith(fontSize: _fontSize, height: _lineHeight),
                ),
              ),
            ),
          ),
        },
      ),
    );
  }

  String _stripTags(String html) {
    final text = html.replaceAll(RegExp(r'<[^>]*>'), ' ');
    return text.replaceAll(RegExp(r'\s+'), ' ').trim();
  }

  Widget _pagedImageBody(ChapterBody_Images urls) {
    return PageView.builder(
      itemCount: urls.field0.length,
      onPageChanged: (index) async {
        final maxExtent = _scroll.position.maxScrollExtent;
        final fraction = maxExtent > 0 ? index / urls.field0.length : 0.0;
        await AppPrefs.instance().then(
          (p) => p.setPosition(current.id, fraction),
        );
      },
      itemBuilder: (context, index) {
        final page = urls.field0[index];
        return page.startsWith('file://')
            ? Center(
                child: Image.file(
                  File.fromUri(Uri.parse(page)),
                  fit: BoxFit.contain,
                ),
              )
            : Center(child: Image.network(page, fit: BoxFit.contain));
      },
    );
  }
}
