import 'package:flutter/material.dart';

import '../service/vault_service.dart';
import 'work_page.dart';

class ContinueReadingRow extends StatefulWidget {
	const ContinueReadingRow({super.key, required this.vault, required this.onChanged});

	final VaultService vault;
	final VoidCallback onChanged;

	@override
	State<ContinueReadingRow> createState() => _ContinueReadingRowState();
}

class _ContinueReadingRowState extends State<ContinueReadingRow> {
	List<ContinueItem>? items;

	@override
	void initState() {
		super.initState();
		_load();
	}

	Future<void> _load() async {
		try {
			final loaded = await widget.vault.continueReading();
			if (!mounted) return;
			setState(() => items = loaded);
		} catch (_) {
			if (mounted) setState(() => items = const []);
		}
	}

	Future<void> _open(ContinueItem item) async {
		if (item.chapterId == null) return;
		final details = await widget.vault.getWork(workId: item.workId);
		final index = details.chapters.indexWhere((chapter) => chapter.id == item.chapterId);
		if (!mounted || index < 0) return;
		await Navigator.of(context).push(MaterialPageRoute(
			builder: (_) => ReaderPage(vault: widget.vault, chapters: details.chapters, index: index),
		));
		await _load();
		widget.onChanged();
	}

	@override
	Widget build(BuildContext context) {
		final shown = items;
		if (shown == null || shown.isEmpty) return const SizedBox.shrink();
		return Column(
			crossAxisAlignment: CrossAxisAlignment.start,
			children: [
				Padding(
					padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
					child:
						Text('CONTINUE READING', style: Theme.of(context).textTheme.labelSmall?.copyWith(letterSpacing: 0.6)),
				),
				SizedBox(
					height: 150,
					child: ListView.builder(
						scrollDirection: Axis.horizontal,
						padding: const EdgeInsets.symmetric(horizontal: 16),
						itemCount: shown.length,
						itemBuilder: (context, index) {
							final item = shown[index];
							return Padding(
								padding: const EdgeInsets.only(right: 12),
								child: InkWell(
									borderRadius: BorderRadius.circular(8),
									onTap: () => _open(item),
									child: SizedBox(
										width: 100,
										child: Column(
											crossAxisAlignment: CrossAxisAlignment.start,
											children: [
												ClipRRect(
													borderRadius: BorderRadius.circular(8),
													child: SizedBox(
														height: 110,
														width: 100,
														child: item.coverUrl != null
																? Image.network(item.coverUrl!, fit: BoxFit.cover)
																: const ColoredBox(color: Color(0xFF322820), child: Icon(Icons.menu_book)),
													),
												),
												const SizedBox(height: 4),
												Text(item.title,
													maxLines: 2,
													overflow: TextOverflow.ellipsis,
													style: Theme.of(context).textTheme.bodySmall),
											],
										),
									),
								),
							);
						},
					),
				),
			],
		);
	}
}
