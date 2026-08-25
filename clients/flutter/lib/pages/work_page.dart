import '../service/vault_events.dart';
import '../service/sync_scheduler.dart';
import 'dart:io';

import 'package:flutter/material.dart';

import '../service/vault_service.dart';

class WorkPage extends StatefulWidget {
	const WorkPage({super.key, required this.vault, required this.details});

	final VaultService vault;
	final WorkDetails details;

	@override
	State<WorkPage> createState() => _WorkPageState();
}

class _WorkPageState extends State<WorkPage> {
	late Future<WorkDetails> _details;
	Set<String> read = {};
	bool newestFirst = false;
	Set<String> downloaded = {};
	bool inLibrary = false;
	bool refreshing = false;
	bool freshChapters = false;
	List<Map<String, dynamic>> trackLinks = const [];
	Set<String> linkedTrackerIds = {};

	@override
	void initState() {
		super.initState();
		VaultEvents.instance.subscribe(_onEvent);
		widget.vault.myTrackerAccounts().then((accounts) {
			if (!mounted) return;
			setState(() => linkedTrackerIds = accounts.map((a) => a['tracker_id'] as String).toSet());
			if (linkedTrackerIds.isNotEmpty) _loadTracks();
		}).catchError((_) {});
		_details = _load();
	}

	@override
	void dispose() {
		VaultEvents.instance.unsubscribe(_onEvent);
		super.dispose();
	}

	void _onEvent(String workId) {
		if (workId != widget.details.id || !mounted) return;
		setState(() => freshChapters = true);
	}

	Future<void> _loadTracks() async {
		final links = await widget.vault.workTracks(workId: widget.details.id).catchError((_) => <Map<String, dynamic>>[]);
		if (mounted) setState(() => trackLinks = links);
	}

	Future<void> _bindTrack(String trackerId, String remoteId) async {
		await widget.vault.bindWorkTrack(
			workId: widget.details.id,
			trackerId: trackerId,
			remoteId: remoteId,
		);
		await _loadTracks();
	}

	Future<void> _unbindTrack(String linkId) async {
		await widget.vault.deleteWorkTrack(workId: widget.details.id, linkId: linkId);
		await _loadTracks();
	}

	Future<void> _refreshTrack(String linkId) async {
		await widget.vault.refreshWorkTrackLink(workId: widget.details.id, linkId: linkId);
		await _loadTracks();
	}

	Future<void> _showTrackDialog() async {
		final remoteId = TextEditingController();
		await showDialog<void>(
			context: context,
			builder: (context) => AlertDialog(
				title: Text('Track on ${linkedTrackerIds.first}'),
				content: Column(
					mainAxisSize: MainAxisSize.min,
					children: [
						for (final link in trackLinks)
							ListTile(
								dense: true,
								title: Text('${link['remote_title']} (${link['tracker_id']})'),
								subtitle: Text('ch. ${link['last_chapters_synced'] ?? 0}'),
								trailing: Row(
									mainAxisSize: MainAxisSize.min,
									children: [
										IconButton(
											icon: const Icon(Icons.refresh, size: 20),
											onPressed: () => _refreshTrack(link['id'] as String),
										),
										IconButton(
											icon: const Icon(Icons.link_off, size: 20),
											onPressed: () => _unbindTrack(link['id'] as String),
										),
									],
								),
							),
						TextField(
							controller: remoteId,
							autofocus: true,
							decoration: const InputDecoration(hintText: 'Remote media id (e.g. 30013)'),
						),
					],
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Close')),
					FilledButton(
						onPressed: () async {
							if (remoteId.text.trim().isEmpty) return;
							try {
								await _bindTrack(linkedTrackerIds.first, remoteId.text.trim());
								if (!context.mounted) return;
								Navigator.of(context).pop();
							} catch (error) {
								if (!context.mounted) return;
								ScaffoldMessenger.of(context)
									.showSnackBar(SnackBar(content: Text('Bind failed: $error')));
							}
						},
						child: const Text('Bind'),
					),
				],
			),
		);
	}

	Future<WorkDetails> _load() async {
		final fresh = await widget.vault.getWork(workId: widget.details.id);
		freshChapters = false;
		read = (await widget.vault.readChapters(workId: fresh.id)).toSet();
		downloaded = (await widget.vault.downloadedChapters(workId: fresh.id).catchError((_) => <String>[])).toSet();
		return fresh;
	}

	Future<void> _toggleLibrary(String workId) async {
		if (inLibrary) {
			await widget.vault.removeFromLibrary(workId: workId);
		} else {
			await widget.vault.addToLibrary(workId: workId);
		}
		setState(() => inLibrary = !inLibrary);
	}

	Future<void> _markPreviousRead(List<ChapterSummary> chapters, int index) async {
		final workId = widget.details.id;
		final pending = [
			for (var i = 0; i <= index; i++)
				if (!read.contains(chapters[i].id)) chapters[i].id,
		];
		if (pending.isEmpty) return;
		await widget.vault.markChapters(workId: workId, chapterIds: pending, read: true);
		if (!mounted) return;
		setState(() => read.addAll(pending));
		SyncScheduler.instance.nudge();
	}

	Future<void> _openChapter(List<ChapterSummary> chapters, int index) async {
		final chapter = chapters[index];
		if (!mounted) return;
		await widget.vault.markRead(chapterId: chapter.id);
		if (!mounted) return;
		setState(() => read.add(chapter.id));
		SyncScheduler.instance.nudge();
		Navigator.of(context)
			.push(MaterialPageRoute(
				builder: (_) => ReaderPage(vault: widget.vault, chapters: chapters, index: index),
			))
			.then((_) => _refresh());
	}

	Future<void> _refreshFromSource() async {
		setState(() => refreshing = true);
		try {
			await widget.vault.refreshWork(workId: widget.details.id);
			await _refresh();
			if (mounted) {
				ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Refreshed from source')));
			}
		} catch (e) {
			if (mounted) {
				ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Refresh failed: $e')));
			}
		} finally {
			if (mounted) setState(() => refreshing = false);
		}
	}

	Future<void> _refresh() async {
		final reload = _load();
		setState(() {
			_details = reload;
		});
		await reload;
	}

	Future<void> _toggleDownload(String chapterId, bool isDownloaded) async {
		if (isDownloaded) {
			await widget.vault.removeDownload(chapterId: chapterId);
		} else {
			try {
				await widget.vault.downloadChapter(chapterId: chapterId);
			} catch (e) {
				if (mounted) {
					ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Download failed: $e')));
				}
				return;
			}
		}
		setState(() {
			isDownloaded ? downloaded.remove(chapterId) : downloaded.add(chapterId);
		});
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			body: FutureBuilder<WorkDetails>(
				future: _details,
				builder: (context, snapshot) {
					if (snapshot.connectionState != ConnectionState.done) {
						return const Center(child: CircularProgressIndicator());
					}
					final details = snapshot.data ?? widget.details;
				final orderedChapters = newestFirst
				? details.chapters.reversed.toList()
				: details.chapters;
				return CustomScrollView(
						slivers: [
							SliverAppBar(
								title: Text(details.title),
								actions: [
									IconButton(
										icon: refreshing
												? const SizedBox(
													width: 18,
													height: 18,
													child: CircularProgressIndicator(strokeWidth: 2),
												)
												: const Icon(Icons.refresh),
										onPressed: refreshing ? null : _refreshFromSource,
										tooltip: 'Check for updates',
									),
									IconButton(
										icon: const Icon(Icons.swap_vert),
										onPressed: () => setState(() => newestFirst = !newestFirst),
										tooltip: newestFirst ? 'Oldest first' : 'Newest first',
									),
									IconButton(
										icon: Icon(inLibrary ? Icons.favorite : Icons.favorite_border),
										onPressed: () => _toggleLibrary(details.id),
									),
								],
							),
							SliverToBoxAdapter(child: _Hero(details: details)),
							SliverToBoxAdapter(
								child: Padding(
									padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
									child: Wrap(
										spacing: 8,
										runSpacing: 8,
										children: [for (final genre in details.genres) Chip(label: Text(genre))],
									),
								),
							),
							const SliverToBoxAdapter(
								child: Padding(
									padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
									child: Text('Chapters', style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, letterSpacing: 0.6)),
								),
							),
							if (linkedTrackerIds.isNotEmpty)
								SliverToBoxAdapter(
									child: Padding(
										padding: const EdgeInsets.symmetric(horizontal: 16),
										child: OutlinedButton.icon(
											icon: const Icon(Icons.auto_stories_outlined),
											label: Text(trackLinks.isEmpty
													? 'Track on ${linkedTrackerIds.first}'
													: 'Tracking (${trackLinks.length})'),
											onPressed: _showTrackDialog,
										),
									),
								),
							if (freshChapters)
								SliverToBoxAdapter(
									child: Padding(
										padding: const EdgeInsets.symmetric(horizontal: 16),
										child: OutlinedButton.icon(
											icon: const Icon(Icons.new_releases_outlined),
											label: const Text('New chapters available'),
											onPressed: () => _refresh(),
										),
									),
								),
							SliverList(
								delegate: SliverChildBuilderDelegate(
									(context, index) {
										final chapter = orderedChapters[index];
										final canonicalIndex = details.chapters.indexOf(chapter);
										final isRead = read.contains(chapter.id);
										final isDownloaded = downloaded.contains(chapter.id);
										return ListTile(
											title: Text(
												chapter.title,
												style: TextStyle(color: isRead ? Theme.of(context).colorScheme.secondary : null),
											),
											trailing: Row(
												mainAxisSize: MainAxisSize.min,
												children: [
												if (widget.vault.supportsDownloads)
													IconButton(
														icon: Icon(
															isDownloaded ? Icons.offline_pin : Icons.download_outlined,
															color: isDownloaded ? Theme.of(context).colorScheme.secondary : null,
														),
														onPressed: () => _toggleDownload(chapter.id, isDownloaded),
													),
													if (isRead)
														const Icon(Icons.check_circle_outline)
													else
														const Icon(Icons.chevron_right),
												],
											),
											onTap: () => _openChapter(details.chapters, canonicalIndex),
											onLongPress: () => showModalBottomSheet<void>(
												context: context,
												builder: (sheetContext) => SafeArea(
													child: Column(
														mainAxisSize: MainAxisSize.min,
														children: [
															ListTile(
																leading: const Icon(Icons.done_all),
																title: Text('Mark previous ${canonicalIndex + 1} as read'),
																onTap: () {
																	Navigator.of(sheetContext).pop();
																	_markPreviousRead(details.chapters, canonicalIndex);
																},
															),
														],
													),
												),
											),
										);
									},
									childCount: details.chapters.length,
								),
							),
						],
					);
				},
			),
		);
	}
}

class _Hero extends StatelessWidget {
	const _Hero({required this.details});

	final WorkDetails details;

	@override
	Widget build(BuildContext context) {
		final scheme = Theme.of(context).colorScheme;
		final label = TextStyle(
			fontFamily: 'Geist',
			fontSize: 12,
			fontWeight: FontWeight.w500,
			letterSpacing: 0.6,
			color: scheme.onSurfaceVariant,
		);
		return Padding(
			padding: const EdgeInsets.all(16),
			child: Row(
				crossAxisAlignment: CrossAxisAlignment.start,
				children: [
					ClipRRect(
						borderRadius: BorderRadius.circular(16),
						child: details.coverUrl != null
								? Image.network(details.coverUrl!, width: 160, height: 240, fit: BoxFit.cover)
								: Container(
									width: 160,
									height: 240,
									color: scheme.surfaceContainerHigh,
									child: const Icon(Icons.menu_book, size: 48),
								),
					),
					const SizedBox(width: 20),
					Expanded(
						child: Column(
							crossAxisAlignment: CrossAxisAlignment.start,
							children: [
								Text(details.title, style: Theme.of(context).textTheme.displaySmall),
								if (details.authors.isNotEmpty) ...[
									const SizedBox(height: 12),
									Text('AUTHOR', style: label),
									Text(details.authors.join(', '), style: Theme.of(context).textTheme.bodyLarge),
								],
								if (details.status != null) ...[
									const SizedBox(height: 12),
									Text('STATUS', style: label),
									Text(details.status!, style: Theme.of(context).textTheme.bodyLarge),
								],
							],
						),
					),
				],
			),
		);
	}
}

class ReaderPage extends StatefulWidget {
	const ReaderPage({super.key, required this.vault, required this.chapters, required this.index});

	final VaultService vault;
	final List<ChapterSummary> chapters;
	final int index;

	@override
	State<ReaderPage> createState() => _ReaderPageState();
}

class _ReaderPageState extends State<ReaderPage> {
	late ChapterSummary current;
	late ChapterBody? body;
	String? translatedHtml;
	List<Map<String, dynamic>> matches = const [];
	String? translationMode;

	@override
	void initState() {
		super.initState();
		current = widget.chapters[widget.index];
		body = null;
		_load(current.id);
		widget.vault.translationMode().then((mode) {
			if (mounted) setState(() => translationMode = mode);
		}).catchError((_) {});
	}

	bool get _canTranslate => translatedHtml == null && translationMode != null && translationMode != 'off' && translationMode != 'unavailable';

	Future<void> _translate() async {
		final target = TextEditingController(text: 'en');
		final source = TextEditingController();
		final confirmed = await showDialog<List<String>>(
			context: context,
			builder: (context) => AlertDialog(
				title: const Text('Translate'),
				content: Column(
					mainAxisSize: MainAxisSize.min,
					children: [
						TextField(controller: target, decoration: const InputDecoration(hintText: 'To (e.g. en)')),
						const SizedBox(height: 12),
						TextField(
							controller: source,
							decoration: const InputDecoration(hintText: 'From (optional, enables glossary)'),
						),
					],
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
					FilledButton(
						onPressed: () => Navigator.of(context).pop([target.text.trim(), source.text.trim()]),
						child: const Text('Translate'),
					),
				],
			),
		);
		if (confirmed == null || confirmed.first.isEmpty) return;
		final result = await widget.vault.translateChapter(
			chapterId: current.id,
			to: confirmed.first,
			from: confirmed.last.isEmpty ? null : confirmed.last,
		);
		if (!mounted) return;
		setState(() {
			translatedHtml = result['content'] as String?;
			matches = (result['matches'] as List?)?.cast<Map<String, dynamic>>() ?? const [];
		});
	}

	void _showGlossary() {
		showModalBottomSheet<void>(
			context: context,
			builder: (context) => SafeArea(
				child: ListView(
					padding: const EdgeInsets.all(16),
					children: [
						for (final entry in matches) ...[
							Text(entry['term'] as String, style: Theme.of(context).textTheme.titleMedium),
							if (entry['romanization'] != null)
								Text(
									entry['romanization'] as String,
									style: Theme.of(context).textTheme.bodySmall?.copyWith(fontStyle: FontStyle.italic),
								),
							for (final meaning in (entry['meanings'] as List).cast<Map<String, dynamic>>())
								ListTile(
									dense: true,
									title: Text(meaning['meaning'] as String),
									trailing: Row(
										mainAxisSize: MainAxisSize.min,
										children: [
											Text('${meaning['votes']}'),
											IconButton(
												icon: Icon(
													meaning['voted_by_me'] == true ? Icons.thumb_up : Icons.thumb_up_outlined,
													size: 18,
												),
												onPressed: () async {
													final voted = await widget.vault.toggleGlossaryVote(meaningId: meaning['id'] as String);
													setState(() => meaning['voted_by_me'] = voted);
													setState(() => meaning['votes'] = (meaning['votes'] as int) + (voted ? 1 : -1));
												},
											),
										],
									),
								),
							const Divider(),
						],
					],
				),
			),
		);
	}

	Future<void> _load(String chapterId) async {
		final content = await widget.vault.chapterContent(chapterId: chapterId);
		await widget.vault.markRead(chapterId: chapterId);
		if (mounted) {
			setState(() {
				body = content;
				translatedHtml = null;
			});
		}
	}

	Future<void> _go(int delta) async {
		final next = widget.index + delta;
		if (next < 0 || next >= widget.chapters.length) return;
		setState(() {
			current = widget.chapters[next];
			body = null;
		});
		await _load(current.id);
	}

	@override
	Widget build(BuildContext context) {
		final scheme = Theme.of(context).colorScheme;
		return Scaffold(
			appBar: AppBar(
				title: Column(
					crossAxisAlignment: CrossAxisAlignment.center,
					children: [
						Text(current.title, overflow: TextOverflow.ellipsis),
						Text(
							'${widget.index + 1} / ${widget.chapters.length}',
							style: TextStyle(fontSize: 11, color: scheme.onSurfaceVariant),
						),
					],
				),
				leading: IconButton(icon: const Icon(Icons.arrow_back), onPressed: () => Navigator.of(context).pop()),
				actions: [
					if (matches.isNotEmpty)
						IconButton(icon: const Icon(Icons.menu_book_outlined), onPressed: _showGlossary),
					if (_canTranslate)
						IconButton(icon: const Icon(Icons.translate), onPressed: _translate),
					IconButton(icon: const Icon(Icons.skip_previous), onPressed: widget.index > 0 ? () => _go(-1) : null),
					IconButton(
						icon: const Icon(Icons.skip_next),
						onPressed: widget.index + 1 < widget.chapters.length ? () => _go(1) : null,
					),
				],
			),
			body: switch (body) {
				null => const Center(child: CircularProgressIndicator()),
				ChapterBody_Images urls => InteractiveViewer(
					child: ListView.builder(
						itemCount: urls.field0.length,
						itemBuilder: (context, index) {
							final page = urls.field0[index];
							return page.startsWith('file://')
									? Image.file(File.fromUri(Uri.parse(page)), fit: BoxFit.cover)
									: Image.network(page);
						},
					),
				),
				ChapterBody_Html html => Center(
					child: ConstrainedBox(
						constraints: const BoxConstraints(maxWidth: 720),
						child: SingleChildScrollView(
							padding: const EdgeInsets.all(24),
							child: Text(
								_stripTags(translatedHtml ?? html.field0),
								style: Theme.of(context).textTheme.bodyLarge,
							),
						),
					),
				),
			},
		);
	}

	String _stripTags(String html) {
		final text = html.replaceAll(RegExp(r'<[^>]*>'), ' ');
		return text.replaceAll(RegExp(r'\s+'), ' ').trim();
	}
}
