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
	Future<Map<String, dynamic>> exportSyncState() async =>
		jsonDecode(await _vault.exportSyncState()) as Map<String, dynamic>;

	@override
	Future<void> applySyncState(Map<String, dynamic> state) =>
		_vault.applySyncState(stateJson: jsonEncode(state));
}
