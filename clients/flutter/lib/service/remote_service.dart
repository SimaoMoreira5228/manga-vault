import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'vault_service.dart';

class RemoteService implements VaultService {
	RemoteService({required this.baseUrl, required this.token});

	final String baseUrl;
	String token;

	Map<String, String> get _auth => {'Authorization': 'Bearer $token'};

	static Future<RemoteService> login({
		required String baseUrl,
		required String username,
		required String password,
	}) async {
		final response = await _requestJson(
			method: 'POST',
			url: '$baseUrl/api/auth/login',
			body: jsonEncode({'username': username, 'password': password, 'device_label': 'Flutter'}),
		);
		return RemoteService(baseUrl: baseUrl, token: response['token'] as String);
	}

	Future<dynamic> _send(String method, String path, {String? body}) => _requestJson(
			method: method,
			url: '$baseUrl$path',
			body: body,
			headers: _auth,
		);

	Future<List<T>> _list<T>(String path, T Function(dynamic) from) async {
		final raw = await _send('GET', path);
		return [for (final item in raw as List) from(item)];
	}

	static Future<dynamic> _requestJson({
		required String method,
		required String url,
		String? body,
		Map<String, String> headers = const {},
	}) async {
		final client = HttpClient();
		try {
			final request = await client.openUrl(method, Uri.parse(url));
			request.headers.set(HttpHeaders.contentTypeHeader, 'application/json');
			headers.forEach(request.headers.set);
			if (body != null) request.write(body);
			final response = await request.close().timeout(const Duration(seconds: 30));
			final text = await utf8.decodeStream(response);
			if (response.statusCode >= 400) throw HttpException('$method $url failed (${response.statusCode}): $text');
			return text.isEmpty ? null : jsonDecode(text);
		} on TimeoutException {
			throw HttpException('$method $url timed out');
		} finally {
			client.close();
		}
	}

	@override
	Future<List<SourceSummary>> listSources() =>
		_list('/api/sources', (raw) => SourceSummary(id: raw['id'], name: raw['name'], version: raw['version'], kind: raw['kind']));

	@override
	Future<List<WorkSummary>> searchSource({required String sourceId, required String query, required int page}) =>
		_list('/api/sources/$sourceId/search?q=$query&page=$page', WorkSummaryMapper.from);

	@override
	Future<List<WorkSummary>> latestSource({required String sourceId, required int page}) =>
		_list('/api/sources/$sourceId/latest?page=$page', WorkSummaryMapper.from);

	@override
	Future<WorkDetails> importWork({required String sourceId, required String remoteUrl}) async {
		final work = await _send('POST', '/api/works', body: jsonEncode({'source_id': sourceId, 'remote_url': remoteUrl}));
		return WorkDetailsMapper.fromWork(work);
	}

	@override
	Future<WorkDetails> getWork({required String workId}) async =>
		WorkDetailsMapper.fromPayload(await _send('GET', '/api/works/$workId'));

	@override
	Future<ChapterBody> chapterContent({required String chapterId}) async {
		final content = await _send('GET', '/api/chapters/$chapterId');
		if (content case {'Html': final html}) return ChapterBody_Html(html as String);
		if (content case {'Images': final pages}) return ChapterBody_Images([for (final page in pages as List) page as String]);
		throw const FormatException('unknown chapter content shape');
	}

	@override
	Future<void> addToLibrary({required String workId}) => _send('PUT', '/api/library', body: jsonEncode({'work_id': workId}));

	@override
	Future<void> removeFromLibrary({required String workId}) => _send('DELETE', '/api/library/$workId');

	@override
	Future<List<LibraryItem>> listLibrary() async {
		final payload = await _send('GET', '/api/library');
		return [
			for (final pair in payload['entries'] as List)
				LibraryItem(entryId: pair[0]['id'] as String, work: WorkDetailsMapper.fromWork(pair[1])),
		];
	}

	@override
	Future<void> markRead({required String chapterId}) => _send('PUT', '/api/chapters/$chapterId/read');

	@override
	Future<List<String>> readChapters({required String workId}) async {
		final payload = await _send('GET', '/api/works/$workId/progress');
		return [for (final id in payload['read_chapter_ids'] as List) id as String];
	}

	@override
	Future<String> translationMode() async {
		final caps = await _send('GET', '/api/me/capabilities');
		return caps['translation']['mode'] as String;
	}

	@override
	Future<void> setTranslationProvider({String? providerBaseUrl, String? apiKey, String? model}) async {
		final payload = {'api_key': ?apiKey, 'base_url': ?providerBaseUrl, 'model': ?model};
		await _send('PUT', '/api/me/translation-settings', body: jsonEncode(payload));
	}

	@override
	Future<void> clearTranslationProvider() => _send('DELETE', '/api/me/translation-settings');

	@override
	Future<String> translateChapter({required String chapterId, required String to}) async {
		final response = await _send(
			'POST',
			'/api/chapters/$chapterId/translate',
			body: jsonEncode({'to': to}),
		);
		return response['content'] as String;
	}

	@override
	Future<Map<String, dynamic>> exportSyncState() async => await _send('GET', '/api/sync/state');

	@override
	Future<void> applySyncState(Map<String, dynamic> state) =>
		_send('POST', '/api/sync/apply', body: jsonEncode(state));

	@override
	Future<void> downloadChapter({required String chapterId}) async =>
		throw UnsupportedError('downloads are device-local');

	@override
	Future<void> removeDownload({required String chapterId}) async =>
		throw UnsupportedError('downloads are device-local');

	@override
	Future<List<String>> downloadedChapters({required String workId}) async => const [];

	@override
	Future<List<PluginRepo>> pluginRepos() =>
		_list('/api/plugin-repos', (raw) => PluginRepo(id: raw['id'], name: raw['name'], url: raw['url']));

	@override
	Future<PluginRepo> addPluginRepo({required String url}) async {
		final repo = await _send('POST', '/api/plugin-repos', body: jsonEncode({'url': url}));
		return PluginRepo(id: repo['id'], name: repo['name'], url: repo['url']);
	}

	@override
	Future<void> removePluginRepo({required String repoId}) => _send('DELETE', '/api/plugin-repos/$repoId');

	@override
	Future<List<CatalogItem>> pluginCatalog() => _list(
		'/api/plugins/catalog',
		(raw) => CatalogItem(
			id: raw['id'],
			backend: raw['backend'],
			repoId: raw['repo_id'],
			repoName: raw['repo_name'],
			availableVersion: raw['available_version'],
			installedVersion: raw['installed_version'],
			updateAvailable: raw['update_available'] as bool,
		),
	);

	@override
	Future<void> installPlugin({required String pluginId}) => _send('PUT', '/api/plugins/$pluginId/install', body: 'null');

	@override
	Future<bool> uninstallPlugin({required String pluginId}) async {
		await _send('DELETE', '/api/plugins/$pluginId');
		return true;
	}
}

class WorkSummaryMapper {
	static WorkSummary from(dynamic raw) => WorkSummary(
		id: null,
		title: raw['title'] as String,
		remoteUrl: raw['remote_url'] as String,
		coverUrl: raw['cover_url'] as String?,
	);
}

class WorkDetailsMapper {
	static WorkDetails fromPayload(dynamic payload) => fromWork(
		payload['work'],
		chapters: payload['chapters'] as List?,
		readIds: [for (final id in payload['read_chapter_ids'] as List) id as String],
	);

	static WorkDetails fromWork(dynamic work, {List<dynamic>? chapters, List<String> readIds = const []}) => WorkDetails(
		id: work['id'] as String,
		kind: work['kind'] as String,
		title: work['title'] as String,
		coverUrl: work['cover_url'] as String?,
		authors: [for (final author in (work['authors'] ?? []) as List) author as String],
		status: work['status'] as String?,
		description: work['description'] as String?,
		genres: [for (final genre in (work['genres'] ?? []) as List) genre as String],
		chapters: [
			for (final chapter in chapters ?? const [])
				ChapterSummary(
					id: chapter['id'] as String,
					title: chapter['title'] as String,
					sortIndex: chapter['sort_index'] as int,
				),
		],
	);
}
