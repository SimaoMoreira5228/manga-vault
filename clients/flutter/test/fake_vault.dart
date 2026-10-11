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
    this.continueItems = const [],
    this.total = 0,
    this.statsError,
    this.trendingError,
    this.clearError,
  });

  final List<SourceSummary> sources;
  final List<WorkSummary> trending;
  ReadingStatsSummary stats;
  List<CategoryInfo> categories;
  final List<LibraryItem> items;
  final List<ContinueItem> continueItems;
  final int total;
  Object? statsError;
  Object? trendingError;
  Object? clearError;

  final created = <String>[];
  final deleted = <String>[];
  final cleared = <String>[];
  final trendingRequests = <({String sourceId, int page})>[];
  final filters = <LibraryFilter>[];

  @override
  String imageUrl(String url) => url;

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
    return trending;
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
    return VaultPage(items: items, total: total, offset: offset, limit: limit);
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
