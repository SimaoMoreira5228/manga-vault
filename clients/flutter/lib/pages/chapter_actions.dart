import 'package:flutter/material.dart';

import '../service/sync_scheduler.dart';
import '../service/vault_service.dart';

Future<void> showChapterActions(
  BuildContext context, {
  required VaultService vault,
  required String workId,
  required List<ChapterSummary> chapters,
  required int index,
  required Future<void> Function() onChanged,
}) => showModalBottomSheet<void>(
  context: context,
  builder: (sheetContext) => SafeArea(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        ListTile(
          leading: const Icon(Icons.done_all),
          title: Text('Mark previous ${index + 1} as read'),
          onTap: () {
            Navigator.of(sheetContext).pop();
            _markPreviousRead(vault, workId, chapters, index, onChanged);
          },
        ),
        ListTile(
          leading: const Icon(Icons.mark_email_unread_outlined),
          title: Text('Mark "${chapters[index].title}" as unread'),
          onTap: () {
            Navigator.of(sheetContext).pop();
            _markUnread(vault, chapters[index], onChanged);
          },
        ),
      ],
    ),
  ),
);

Future<void> _markPreviousRead(
  VaultService vault,
  String workId,
  List<ChapterSummary> chapters,
  int index,
  Future<void> Function() onChanged,
) async {
  await vault.markChapters(
    workId: workId,
    chapterIds: [for (var i = 0; i <= index; i++) chapters[i].id],
    read: true,
  );
  SyncScheduler.instance.nudge();
  await onChanged();
}

Future<void> _markUnread(
  VaultService vault,
  ChapterSummary chapter,
  Future<void> Function() onChanged,
) async {
  await vault.markUnread(chapterId: chapter.id);
  SyncScheduler.instance.nudge();
  await onChanged();
}
