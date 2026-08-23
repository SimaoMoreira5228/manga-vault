import '../src/rust/api/local.dart';

export '../src/rust/api/local.dart';

abstract class VaultService {
	Future<List<SourceSummary>> listSources();
	Future<List<WorkSummary>> searchSource({required String sourceId, required String query, required int page});
	Future<List<WorkSummary>> latestSource({required String sourceId, required int page});
	Future<WorkDetails> importWork({required String sourceId, required String remoteUrl});
	Future<WorkDetails> getWork({required String workId});
	Future<ChapterBody> chapterContent({required String chapterId});
	Future<void> addToLibrary({required String workId});
	Future<void> removeFromLibrary({required String workId});
	Future<List<LibraryItem>> listLibrary();
	Future<void> markRead({required String chapterId});
	Future<List<String>> readChapters({required String workId});
	Future<String> translationMode();
	Future<void> setTranslationProvider({String? providerBaseUrl, String? apiKey, String? model});
	Future<void> clearTranslationProvider();
	Future<String> translateChapter({required String chapterId, required String to});
	Future<Map<String, dynamic>> exportSyncState();
	Future<void> applySyncState(Map<String, dynamic> state);
	bool get supportsDownloads;

	Future<void> downloadChapter({required String chapterId});
	Future<void> removeDownload({required String chapterId});
	Future<List<String>> downloadedChapters({required String workId});
	Future<List<PluginRepo>> pluginRepos();
	Future<PluginRepo> addPluginRepo({required String url});
	Future<void> removePluginRepo({required String repoId});
	Future<List<CatalogItem>> pluginCatalog();
	Future<void> installPlugin({required String pluginId});
	Future<bool> uninstallPlugin({required String pluginId});
}
