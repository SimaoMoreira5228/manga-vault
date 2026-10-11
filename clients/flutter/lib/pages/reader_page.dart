import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';

import '../service/app_prefs.dart';
import '../service/vault_service.dart';
class ReaderPage extends StatefulWidget {
  const ReaderPage({
    super.key,
    required this.vault,
    required this.chapters,
    required this.index,
    required this.workId,
    this.page,
  });

  final VaultService vault;
  final List<ChapterSummary> chapters;
  final int index;
  final String workId;
  final int? page;

  @override
  State<ReaderPage> createState() => _ReaderPageState();
}

class _ReaderPageState extends State<ReaderPage> {
  late ChapterSummary current;
  late ChapterBody? body;
  late int _index;
  final ScrollController _scroll = ScrollController();
  final PageController _pageController = PageController();
  final Map<String, int> _positions = {};
  Future<void>? _positionsLoaded;
  Timer? _saveTimer;
  int _currentPage = 0;
  int _resumePage = 0;
  int _entryPage = 0;
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
    final next = _index + 1;
    return next >= 0 && next < widget.chapters.length
        ? widget.chapters[next]
        : null;
  }

  int get _pageCount => switch (body) {
    ChapterBody_Images urls => urls.field0.length,
    _ => 0,
  };

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

    // NOTE: continuous mode has no per-item offsets, so the page is derived
    // from the scroll fraction. Exact per-page tracking needs item extents.
    final page = _pageCount < 2 ? 0 : (fraction * (_pageCount - 1)).round() + 1;
    if (page != _currentPage) {
      _currentPage = page;
      _queuePositionSave();
    }
  }

  void _queuePositionSave() {
    _saveTimer?.cancel();
    _saveTimer = Timer(const Duration(seconds: 2), _flushPositionSave);
  }

  Future<void> _flushPositionSave() async {
    _saveTimer?.cancel();
    _saveTimer = null;
    if (_currentPage <= 0) return;
    final chapterId = current.id;
    final page = _currentPage;
    try {
      await widget.vault.savePosition(chapterId: chapterId, page: page);
    } catch (_) {}
  }

  void _resumeSavedPosition() {
    final page = _resumePage;
    if (page > 1 && page <= _pageCount && _pagedMode && _pageController.hasClients) {
      _pageController.jumpToPage(page - 1);
      return;
    }
    final fraction = _resumeFraction;
    if (fraction == null || !_scroll.hasClients) return;
    final maxExtent = _scroll.position.maxScrollExtent;
    if (maxExtent > 0) _scroll.jumpTo(maxExtent * fraction);
  }

  String? translatedHtml;
  List<Map<String, dynamic>> matches = const [];
  String? translationMode;

  @override
  void initState() {
    super.initState();
    _index = widget.index;
    current = widget.chapters[_index];
    _workId = widget.workId;
    _entryPage = widget.page ?? 0;
    body = null;
    _scroll.addListener(_onScroll);
    _loadSettings();
    _positionsLoaded = _loadPositions();
    _load(current.id);
    widget.vault
        .translationMode()
        .then((mode) {
          if (mounted) setState(() => translationMode = mode);
        })
        .catchError((_) {});
  }

  Future<void> _loadPositions() async {
    final positions = await widget.vault
        .positionsForWork(workId: _workId)
        .catchError((_) => <ReadingPosition>[]);
    for (final position in positions) {
      _positions[position.chapterId] = position.page;
    }
  }

  @override
  void dispose() {
    _flushPositionSave();
    _scroll.dispose();
    _pageController.dispose();
    super.dispose();
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
    _currentPage = 0;

    final prefs = await AppPrefs.instance();
    await _positionsLoaded;
    final page = switch (_entryPage > 0 ? _entryPage : _positions[chapterId]) {
      final p? when _pageCount > 1 => p,
      _ => 0,
    };
    _entryPage = 0;
    _resumePage = page;
    _resumeFraction = page > 1
        ? (page - 1) / (_pageCount - 1)
        : prefs.positionFor(chapterId);
    final fraction = _resumeFraction;
    if (_resumePage > 1 || (fraction != null && fraction > 0.03)) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              _resumePage > 1
                  ? 'Resume at page $_resumePage'
                  : 'Resume at ${(fraction! * 100).round()}%',
            ),
            action: SnackBarAction(
              label: 'Resume',
              onPressed: _resumeSavedPosition,
            ),
          ),
        );
      });
    }
  }

  Future<void> _go(int delta) async {
    final next = _index + delta;
    if (next < 0 || next >= widget.chapters.length) return;
    await _flushPositionSave();
    setState(() {
      _index = next;
      current = widget.chapters[next];
      body = null;
    });
    await _load(current.id);
    if (_pagedMode && _pageController.hasClients) {
      _pageController.jumpToPage(0);
    }
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
              '${_index + 1} / ${widget.chapters.length}',
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
            onPressed: _index > 0 ? () => _go(-1) : null,
          ),
          IconButton(
            icon: const Icon(Icons.skip_next),
            onPressed: _index + 1 < widget.chapters.length
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
                            : Image.network(
                                widget.vault.imageUrl(page),
                                headers: widget.vault.imageHeaders,
                                fit: BoxFit.fitWidth,
                              );
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
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: _htmlBlocks(
                    context,
                    translatedHtml ?? html.field0,
                  ),
                ),
              ),
            ),
          ),
        },
      ),
    );
  }

  List<Widget> _htmlBlocks(BuildContext context, String html) {
    final blocks = <Widget>[];
    final blockPattern = RegExp(
      r'<(p|h[1-6]|li|blockquote|figcaption)\b[^>]*>(.*?)</\1>'
      r'|<img\b[^>]*\bsrc\s*=\s*["\x27]([^"\x27]+)["\x27][^>]*>'
      r'|<hr\s*/?>',
      caseSensitive: false,
      dotAll: true,
    );

    void addText(String raw, {bool heading = false}) {
      final text = _cleanInline(raw);
      if (text.isEmpty) return;
      blocks.add(
        Padding(
          padding: EdgeInsets.only(bottom: heading ? 12 : 16),
          child: Text(
            text,
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
              fontSize: heading ? _fontSize + 2 : _fontSize,
              height: _lineHeight,
              fontWeight: heading ? FontWeight.w600 : null,
            ),
          ),
        ),
      );
    }

    var last = 0;
    for (final match in blockPattern.allMatches(html)) {
      if (match.start > last) addText(html.substring(last, match.start));
      final src = match.group(3);
      if (src != null && src.trim().isNotEmpty) {
        blocks.add(
          Padding(
            padding: EdgeInsets.only(bottom: _imageGap),
            child: Image.network(
              widget.vault.imageUrl(src.trim()),
              headers: widget.vault.imageHeaders,
              fit: BoxFit.fitWidth,
            ),
          ),
        );
      } else {
        final tag = match.group(1)?.toLowerCase() ?? '';
        addText(match.group(2) ?? '', heading: tag.startsWith('h'));
      }
      last = match.end;
    }
    if (last < html.length) addText(html.substring(last));
    return blocks;
  }

  String _cleanInline(String html) {
    final text = html
        .replaceAll(RegExp(r'<br\s*/?>', caseSensitive: false), '\n')
        .replaceAll(RegExp(r'<[^>]*>'), '');
    return text
        .replaceAll('&nbsp;', ' ')
        .replaceAll('&amp;', '&')
        .replaceAll('&lt;', '<')
        .replaceAll('&gt;', '>')
        .replaceAll('&quot;', '"')
        .replaceAll('&#39;', "'")
        .replaceAll('&#x27;', "'")
        .replaceAll(RegExp(r'[ \t]+'), ' ')
        .trim();
  }

  Widget _pagedImageBody(ChapterBody_Images urls) {
    return PageView.builder(
      controller: _pageController,
      itemCount: urls.field0.length,
      onPageChanged: (index) async {
        _currentPage = index + 1;
        _queuePositionSave();
        final prefs = await AppPrefs.instance();
        await prefs.setPosition(
          current.id,
          urls.field0.isEmpty ? 0.0 : index / urls.field0.length,
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
            : Center(
                child: Image.network(
                  widget.vault.imageUrl(page),
                  headers: widget.vault.imageHeaders,
                  fit: BoxFit.contain,
                ),
              );
      },
    );
  }
}

