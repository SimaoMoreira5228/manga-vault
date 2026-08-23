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
	bool inLibrary = false;

	@override
	void initState() {
		super.initState();
		_details = _load();
	}

	Future<WorkDetails> _load() async {
		final fresh = await widget.vault.getWork(workId: widget.details.id);
		read = (await widget.vault.readChapters(workId: fresh.id)).toSet();
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

	Future<void> _openChapter(List<ChapterSummary> chapters, int index) async {
		final chapter = chapters[index];
		if (!mounted) return;
		await widget.vault.markRead(chapterId: chapter.id);
		if (!mounted) return;
		setState(() => read.add(chapter.id));
		Navigator.of(context)
			.push(MaterialPageRoute(
				builder: (_) => ReaderPage(vault: widget.vault, chapters: chapters, index: index),
			))
			.then((_) => _refresh());
	}

	Future<void> _refresh() async {
		final reload = _load();
		setState(() {
			_details = reload;
		});
		await reload;
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
					return CustomScrollView(
						slivers: [
							SliverAppBar(
								title: Text(details.title),
								actions: [
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
							SliverList(
								delegate: SliverChildBuilderDelegate(
									(context, index) {
										final chapter = details.chapters[index];
										final isRead = read.contains(chapter.id);
										return ListTile(
											title: Text(
												chapter.title,
												style: TextStyle(color: isRead ? Theme.of(context).colorScheme.secondary : null),
											),
											trailing: isRead
													? const Icon(Icons.check_circle_outline)
													: const Icon(Icons.chevron_right),
											onTap: () => _openChapter(details.chapters, index),
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

	@override
	void initState() {
		super.initState();
		current = widget.chapters[widget.index];
		body = null;
		_load(current.id);
	}

	Future<void> _load(String chapterId) async {
		final content = await widget.vault.chapterContent(chapterId: chapterId);
		await widget.vault.markRead(chapterId: chapterId);
		if (mounted) setState(() => body = content);
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
						itemBuilder: (context, index) => Image.network(urls.field0[index]),
					),
				),
				ChapterBody_Html html => Center(
					child: ConstrainedBox(
						constraints: const BoxConstraints(maxWidth: 720),
						child: SingleChildScrollView(
							padding: const EdgeInsets.all(24),
							child: Text(_stripTags(html.field0), style: Theme.of(context).textTheme.bodyLarge),
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
