import '../src/rust/api/local.dart';

export '../src/rust/api/local.dart' hide HistoryPage, LibraryPage;
export 'library_filter.dart';

final Map<String, ChapterBody> _chapterContentCache = {};

const int pageSize = 50;
const int maxPageSize = 200;

class VaultPage<T> {
  const VaultPage({
    required this.items,
    required this.total,
    required this.offset,
    required this.limit,
  });
  final List<T> items;
  final int total;
  final int offset;
  final int limit;

  int get number => offset ~/ limit + 1;
  int get pages => limit < 1 ? 1 : (total + limit - 1) ~/ limit;
  bool get hasNext => offset + limit < total;
  bool get hasPrevious => offset > 0;
}

extension VaultLibraryPages on VaultService {
  Future<List<LibraryItem>> allLibrary() async {
    final all = <LibraryItem>[];
    var offset = 0;
    var total = 1;
    while (offset < total) {
      final page = await listLibrary(offset: offset, limit: maxPageSize);
      all.addAll(page.items);
      total = page.total;
      offset = page.offset + page.limit;
    }
    return all;
  }
}

ChapterBody? cachedChapterContent(String chapterId) =>
    _chapterContentCache[chapterId];

void stashChapterContent(String chapterId, ChapterBody body) {
  if (_chapterContentCache.length >= 6) {
    _chapterContentCache.remove(_chapterContentCache.keys.first);
  }
  _chapterContentCache[chapterId] = body;
}

class ChapterProgressStats {
  const ChapterProgressStats({required this.read, required this.total});
  final int read;
  final int total;
}

class ReadingStatsSummary {
  const ReadingStatsSummary({
    required this.totalRead,
    required this.streak,
    required this.worksStarted,
    required this.daily,
  });
  final int totalRead;
  final int streak;
  final int worksStarted;
  final Map<String, int> daily;
}

class HistoryEntry {
  const HistoryEntry({
    required this.readAt,
    required this.chapterId,
    required this.chapterTitle,
    required this.workId,
    required this.workTitle,
  });
  final DateTime? readAt;
  final String chapterId;
  final String? chapterTitle;
  final String workId;
  final String workTitle;
}

class CategoryInfo {
  const CategoryInfo({required this.id, required this.name});
  final String id;
  final String name;
}

class MigrationCandidateInfo {
  const MigrationCandidateInfo({required this.title, required this.remoteUrl});
  final String title;
  final String remoteUrl;
}

class MigrationPairResult {
  const MigrationPairResult({required this.fromWorkId, required this.toWorkId});
  final String fromWorkId;
  final String? toWorkId;
}

class SessionInfo {
  const SessionInfo({
    required this.token,
    required this.deviceLabel,
    required this.lastSeenAt,
  });
  final String token;
  final String? deviceLabel;
  final DateTime lastSeenAt;
}

abstract class VaultService {
  String imageUrl(String url, {String? referer});

  Map<String, String> get imageHeaders;

  Future<List<SourceSummary>> listSources();
  Future<List<WorkSummary>> searchSource({
    required String sourceId,
    required String query,
    required int page,
  });
  Future<List<WorkSummary>> latestSource({
    required String sourceId,
    required int page,
  });
  Future<List<WorkSummary>> trendingSource({
    required String sourceId,
    required int page,
  });
  Future<WorkDetails> importWork({
    required String sourceId,
    required String remoteUrl,
  });
  Future<WorkDetails> getWork({required String workId});
  Future<ChapterBody> chapterContent({required String chapterId});
  Future<void> addToLibrary({required String workId});
  Future<void> removeFromLibrary({required String workId});
  Future<VaultPage<LibraryItem>> listLibrary({
    int offset = 0,
    int limit = pageSize,
    LibraryFilter filter = const LibraryFilter(),
  });
  Future<Map<String, ChapterProgressStats>> libraryOverview();
  Future<VaultPage<HistoryEntry>> history({int offset, int limit});
  Future<ReadingStatsSummary> readingStats();
  Future<int> refreshAllLibrary();
  Future<List<CategoryInfo>> listCategories();
  Future<void> createCategory({required String name});
  Future<void> deleteCategory({required String categoryId});
  Future<void> setEntryCategory({required String entryId, String? categoryId});
  Future<List<MigrationCandidateInfo>> migrationCandidates({
    required String workId,
    required String toSource,
  });
  Future<List<MigrationPairResult>> migrationApply({
    required String toSource,
    required List<(String, String)> pairs,
  });
  Future<String> exportBackup();
  Future<void> importBackup(String json);
  Future<void> markRead({required String chapterId});
  Future<void> markUnread({required String chapterId});
  Future<int> markChapters({
    required String workId,
    required List<String> chapterIds,
    required bool read,
  });
  Future<List<String>> readChapters({required String workId});
  Future<void> savePosition({required String chapterId, required int page});
  Future<bool> clearPosition({required String chapterId});
  Future<List<ReadingPosition>> positionsForWork({required String workId});
  Future<List<ContinueItem>> continueReading();
  Future<void> refreshWork({required String workId});
  Future<String> translationMode();
  Future<void> setTranslationProvider({
    String? providerBaseUrl,
    String? apiKey,
    String? model,
  });
  Future<void> clearTranslationProvider();
  Future<Map<String, dynamic>> translateChapter({
    required String chapterId,
    required String to,
    String? from,
  });
  Future<List<Map<String, dynamic>>> glossaryForLanguage({
    required String language,
  });
  Future<void> createGlossaryEntry({
    required String term,
    required String language,
    required String meaning,
    String? romanization,
  });
  Future<void> addGlossaryMeaning({
    required String entryId,
    required String meaning,
  });
  Future<bool> toggleGlossaryVote({required String meaningId});
  Future<Map<String, dynamic>> exportSyncState();
  Future<void> applySyncState(Map<String, dynamic> state);
  bool get supportsDownloads;
  bool get supportsTrackers;
  Future<List<Map<String, dynamic>>> trackersRegistry();
  Future<List<Map<String, dynamic>>> myTrackerAccounts();
  Future<void> linkTracker({
    required String trackerId,
    String? token,
    String? username,
    String? password,
  });
  Future<void> unlinkTracker({required String trackerId});
  Future<String> startTrackerOauth({
    required String trackerId,
    required String redirectUri,
  });
  Future<List<Map<String, dynamic>>> workTracks({required String workId});
  Future<Map<String, dynamic>> bindWorkTrack({
    required String workId,
    required String trackerId,
    required String remoteId,
  });
  Future<void> deleteWorkTrack({
    required String workId,
    required String linkId,
  });
  Future<Map<String, dynamic>> refreshWorkTrackLink({
    required String workId,
    required String linkId,
  });
  Future<void> downloadChapter({required String chapterId});
  Future<void> removeDownload({required String chapterId});
  Future<List<String>> downloadedChapters({required String workId});
  Future<List<PluginRepo>> pluginRepos();
  Future<PluginRepo> addPluginRepo({required String url});
  Future<void> removePluginRepo({required String repoId});
  Future<List<CatalogItem>> pluginCatalog();
  Future<void> installPlugin({required String pluginId, String? repoId});
  Future<bool> uninstallPlugin({required String pluginId});
}
