import 'vault_service.dart';

class WorkSummaryMapper {
  static WorkSummary from(String sourceId, dynamic raw) => WorkSummary(
    id: null,
    sourceId: sourceId,
    title: raw['title'] as String,
    remoteUrl: raw['remote_url'] as String,
    coverUrl: raw['cover_url'] as String?,
  );
}

class WorkDetailsMapper {
  static WorkDetails fromPayload(dynamic payload) => fromWork(
    payload['work'],
    chapters: payload['chapters'] as List?,
    readIds: [
      for (final id in payload['read_chapter_ids'] as List) id as String,
    ],
  );

  static WorkDetails fromWork(
    dynamic work, {
    List<dynamic>? chapters,
    List<String> readIds = const [],
  }) => WorkDetails(
    id: work['id'] as String,
    sourceId: work['source_id'] as String,
    remoteUrl: work['remote_url'] as String,
    kind: work['kind'] as String,
    title: work['title'] as String,
    coverUrl: work['cover_url'] as String?,
    authors: [
      for (final author in (work['authors'] ?? []) as List) author as String,
    ],
    status: work['status'] as String?,
    description: work['description'] as String?,
    genres: [
      for (final genre in (work['genres'] ?? []) as List) genre as String,
    ],
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
