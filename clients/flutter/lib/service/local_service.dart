import 'dart:convert';

import 'vault_service.dart';

class LocalService implements VaultService {
	LocalService(this._vault);

	final LocalVault _vault;

	@override
	Future<List<SourceSummary>> listSources() => _vault.listSources();

	@override
	Future<List<WorkSummary>> searchSource({required String sourceId, required String query, required int page}) =>
		_vault.searchSource(sourceId: sourceId, query: query, page: page);

	@override
	Future<List<WorkSummary>> latestSource({required String sourceId, required int page}) =>
		_vault.latestSource(sourceId: sourceId, page: page);

	@override
	Future<WorkDetails> importWork({required String sourceId, required String remoteUrl}) =>
		_vault.importWork(sourceId: sourceId, remoteUrl: remoteUrl);

	@override
	Future<WorkDetails> getWork({required String workId}) => _vault.getWork(workId: workId);

	@override
	Future<ChapterBody> chapterContent({required String chapterId}) => _vault.chapterContent(chapterId: chapterId);

	@override
	Future<void> addToLibrary({required String workId}) => _vault.addToLibrary(workId: workId);

	@override
	Future<void> removeFromLibrary({required String workId}) => _vault.removeFromLibrary(workId: workId);

	@override
	Future<List<LibraryItem>> listLibrary() => _vault.listLibrary();

	@override
	Future<void> markRead({required String chapterId}) => _vault.markRead(chapterId: chapterId);

	@override
	Future<List<String>> readChapters({required String workId}) => _vault.readChapters(workId: workId);

	@override
	Future<List<ContinueItem>> continueReading() => _vault.continueReading();

	@override
	Future<void> refreshWork({required String workId}) => _vault.refreshWork(workId: workId);

	@override
	bool get supportsDownloads => true;

	@override
	bool get supportsTrackers => false;

	@override
	Future<List<Map<String, dynamic>>> trackersRegistry() async => const [];

	@override
	Future<List<Map<String, dynamic>>> myTrackerAccounts() async => const [];

	@override
	Future<void> linkTracker({
		required String trackerId,
		String? token,
		String? username,
		String? password,
	}) async => throw UnsupportedError('trackers require a server');

	@override
	Future<void> unlinkTracker({required String trackerId}) async =>
		throw UnsupportedError('trackers require a server');

	@override
	Future<String> startTrackerOauth({required String trackerId, required String redirectUri}) async =>
		throw UnsupportedError('trackers require a server');

	@override
	Future<List<Map<String, dynamic>>> workTracks({required String workId}) async => const [];

	@override
	Future<Map<String, dynamic>> bindWorkTrack({
		required String workId,
		required String trackerId,
		required String remoteId,
	}) async =>
		throw UnsupportedError('trackers require a server');

	@override
	Future<void> deleteWorkTrack({required String workId, required String linkId}) async {}

	@override
	Future<Map<String, dynamic>> refreshWorkTrackLink({required String workId, required String linkId}) async =>
		throw UnsupportedError('trackers require a server');

	@override
	Future<void> downloadChapter({required String chapterId}) => _vault.downloadChapter(chapterId: chapterId);

	@override
	Future<void> removeDownload({required String chapterId}) => _vault.removeDownload(chapterId: chapterId);

	@override
	Future<List<String>> downloadedChapters({required String workId}) => _vault.downloadedChapters(workId: workId);

	@override
	Future<List<PluginRepo>> pluginRepos() => _vault.pluginRepos();

	@override
	Future<PluginRepo> addPluginRepo({required String url}) => _vault.addPluginRepo(url: url);

	@override
	Future<void> removePluginRepo({required String repoId}) => _vault.removePluginRepo(repoId: repoId);

	@override
	Future<List<CatalogItem>> pluginCatalog() => _vault.pluginCatalog();

	@override
	Future<void> installPlugin({required String pluginId}) => _vault.installPlugin(pluginId: pluginId);

	@override
	Future<bool> uninstallPlugin({required String pluginId}) => _vault.uninstallPlugin(pluginId: pluginId);

	@override
	Future<String> translationMode() => _vault.translationMode();

	@override
	Future<void> setTranslationProvider({String? providerBaseUrl, String? apiKey, String? model}) =>
		_vault.setTranslationProvider(
			providerBaseUrl: providerBaseUrl,
			apiKey: apiKey,
			model: model,
		);

	@override
	Future<void> clearTranslationProvider() => _vault.clearTranslationProvider();

	@override
	Future<Map<String, dynamic>> translateChapter({
		required String chapterId,
		required String to,
		String? from,
	}) async {
		final payload = await _vault.translateChapter(chapterId: chapterId, to: to, from: from);
		return jsonDecode(payload) as Map<String, dynamic>;
	}

	@override
	Future<List<Map<String, dynamic>>> glossaryForLanguage({required String language}) async =>
		(jsonDecode(await _vault.glossaryForLanguage(language: language)) as List)
			.cast<Map<String, dynamic>>();

	@override
	Future<void> createGlossaryEntry({
		required String term,
		required String language,
		required String meaning,
		String? romanization,
	}) =>
		_vault.createGlossaryEntry(
			term: term,
			language: language,
			meaning: meaning,
			romanization: romanization,
		);

	@override
	Future<void> addGlossaryMeaning({required String entryId, required String meaning}) =>
		_vault.addGlossaryMeaning(entryId: entryId, meaning: meaning);

	@override
	Future<bool> toggleGlossaryVote({required String meaningId}) =>
		_vault.toggleGlossaryVote(meaningId: meaningId);

	@override
	Future<Map<String, dynamic>> exportSyncState() async =>
		jsonDecode(await _vault.exportSyncState()) as Map<String, dynamic>;

	@override
	Future<void> applySyncState(Map<String, dynamic> state) =>
		_vault.applySyncState(stateJson: jsonEncode(state));
}
