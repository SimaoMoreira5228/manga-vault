import 'dart:async';

import 'package:manga_vault/service/vault_service.dart';

class FakeVault implements VaultService {
  FakeVault({
    this.sources = const [],
    this.trending = const [],
    this.stats = const ReadingStatsSummary(
      totalRead: 0,
      streak: 0,
      worksStarted: 0,
      daily: {},
    ),
    this.categories = const [],
    this.items = const [],
    this.reading = const [],
    this.continueItems = const [],
    this.total,
    this.statsError,
    this.trendingError,
    this.clearError,
    Map<String, WorkDetails>? works,
    Map<String, ChapterBody>? contents,
  }) : works = works ?? {},
       contents = contents ?? {};

  final List<SourceSummary> sources;
  final List<WorkSummary> trending;
  ReadingStatsSummary stats;
  List<CategoryInfo> categories;
  final List<LibraryItem> items;
  final List<HistoryEntry> reading;
  final List<ContinueItem> continueItems;
  final int? total;
  Object? statsError;
  Object? trendingError;
  Object? latestError;
  Object? clearError;

  final Map<String, WorkDetails> works;
  final Map<String, ChapterBody> contents;
  final positions = <String, List<ReadingPosition>>{};
  final imageUrls = <String>[];
  final imports = <({String sourceId, String remoteUrl})>[];
  Object? importError;
  Completer<void>? importGate;

  @override
  Future<WorkDetails> importWork({
    required String sourceId,
    required String remoteUrl,
  }) async {
    imports.add((sourceId: sourceId, remoteUrl: remoteUrl));
    final gate = importGate;
    if (gate != null) await gate.future;
    final failure = importError;
    if (failure != null) throw failure;
    return WorkDetails(
      id: 'imported',
      sourceId: sourceId,
      remoteUrl: remoteUrl,
      kind: 'manga',
      title: 'Imported Work',
      authors: const [],
      genres: const [],
      chapters: const [],
    );
  }

  @override
  Future<WorkDetails> getWork({required String workId}) async =>
      works[workId] ??
      (throw StateError('no fake work $workId'));

  @override
  Future<List<ReadingPosition>> positionsForWork({required String workId}) async =>
      positions[workId] ?? const [];

  @override
  Future<ChapterBody> chapterContent({required String chapterId}) async =>
      contents[chapterId] ?? const ChapterBody.images([]);

  @override
  Future<String> translationMode() async => 'off';

  @override
  Future<void> markRead({required String chapterId}) async {}

  @override
  Future<void> savePosition({required String chapterId, required int page}) async {}

  @override
  Future<Map<String, dynamic>> translateChapter({
    required String chapterId,
    required String to,
    String? from,
  }) async => const {};

  @override
  Future<bool> toggleGlossaryVote({required String meaningId}) async => false;

  final created = <String>[];
  final deleted = <String>[];
  final cleared = <String>[];
  final trendingRequests = <({String sourceId, int page})>[];
  final latestRequests = <({String sourceId, int page})>[];
  final libraryOffsets = <int>[];
  final historyOffsets = <int>[];
  final filters = <LibraryFilter>[];

  @override
  String imageUrl(String url, {String? referer}) {
    imageUrls.add(url);
    return referer == null || referer.isEmpty ? url : '$url?referer=$referer';
  }

  @override
  Map<String, String> get imageHeaders => const {};

  @override
  Future<List<SourceSummary>> listSources() async => sources;

  @override
  Future<List<WorkSummary>> trendingSource({
    required String sourceId,
    required int page,
  }) async {
    trendingRequests.add((sourceId: sourceId, page: page));
    if (trendingError != null) throw trendingError!;
    return _slice(trending, page);
  }

  @override
  Future<List<WorkSummary>> latestSource({
    required String sourceId,
    required int page,
  }) async {
    latestRequests.add((sourceId: sourceId, page: page));
    if (latestError != null) throw latestError!;
    return _slice(trending, page);
  }

  @override
  Future<ReadingStatsSummary> readingStats() async {
    if (statsError != null) throw statsError!;
    return stats;
  }

  @override
  Future<List<CategoryInfo>> listCategories() async => categories;

  @override
  Future<void> createCategory({required String name}) async {
    created.add(name);
    categories = [
      ...categories,
      CategoryInfo(id: 'c${categories.length}', name: name),
    ];
  }

  @override
  Future<void> deleteCategory({required String categoryId}) async {
    deleted.add(categoryId);
    categories = categories
        .where((category) => category.id != categoryId)
        .toList();
  }

  @override
  Future<VaultPage<LibraryItem>> listLibrary({
    int offset = 0,
    int limit = pageSize,
    LibraryFilter filter = const LibraryFilter(),
  }) async {
    filters.add(filter);
    libraryOffsets.add(offset);
    return VaultPage(
      items: _window(items, offset, limit),
      total: total ?? items.length,
      offset: offset,
      limit: limit,
    );
  }

  @override
  Future<VaultPage<HistoryEntry>> history({
    int offset = 0,
    int limit = pageSize,
  }) async {
    historyOffsets.add(offset);
    return VaultPage(
      items: _window(reading, offset, limit),
      total: total ?? reading.length,
      offset: offset,
      limit: limit,
    );
  }

  @override
  Future<Map<String, ChapterProgressStats>> libraryOverview() async => {};

  @override
  Future<List<ContinueItem>> continueReading() async => [
    for (final item in continueItems)
      if (cleared.contains(item.chapterId))
        ContinueItem(
          workId: item.workId,
          title: item.title,
          coverUrl: item.coverUrl,
          remoteUrl: item.remoteUrl,
          chapterId: item.chapterId,
        )
      else
        item,
  ];

  @override
  Future<bool> clearPosition({required String chapterId}) async {
    if (clearError != null) throw clearError!;
    cleared.add(chapterId);
    return true;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

List<T> _window<T>(List<T> all, int offset, int limit) {
  if (offset >= all.length) return const [];
  final end = offset + limit;
  return all.sublist(offset, end > all.length ? all.length : end);
}

List<T> _slice<T>(List<T> all, int page) =>
    _window(all, (page - 1) * pageSize, pageSize);
