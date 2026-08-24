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
	Future<List<ContinueItem>> continueReading();
	Future<void> refreshWork({required String workId});
	Future<String> translationMode();
	Future<void> setTranslationProvider({String? providerBaseUrl, String? apiKey, String? model});
	Future<void> clearTranslationProvider();
	Future<Map<String, dynamic>> translateChapter({
		required String chapterId,
		required String to,
		String? from,
	});
	Future<List<Map<String, dynamic>>> glossaryForLanguage({required String language});
	Future<void> createGlossaryEntry({
		required String term,
		required String language,
		required String meaning,
		String? romanization,
	});
	Future<void> addGlossaryMeaning({required String entryId, required String meaning});
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
	Future<String> startTrackerOauth({required String trackerId, required String redirectUri});
	Future<List<Map<String, dynamic>>> workTracks({required String workId});
	Future<Map<String, dynamic>> bindWorkTrack({
		required String workId,
		required String trackerId,
		required String remoteId,
	});
	Future<void> deleteWorkTrack({required String workId, required String linkId});
	Future<Map<String, dynamic>> refreshWorkTrackLink({required String workId, required String linkId});
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
