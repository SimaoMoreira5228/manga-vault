import 'package:flutter/material.dart';

import '../service/sync_scheduler.dart';
import '../service/vault_service.dart';
import 'continue_reading.dart';
import 'work_page.dart';

class LibraryPage extends StatefulWidget {
	const LibraryPage({super.key, required this.vault});

	final VaultService vault;

	@override
	State<LibraryPage> createState() => _LibraryPageState();
}

class _LibraryPageState extends State<LibraryPage> {
	List<LibraryItem>? items;

	@override
	void initState() {
		super.initState();
		_load();
	}

	Future<void> _load() async {
		final loaded = await widget.vault.listLibrary();
		if (!mounted) return;
		setState(() => items = loaded);
	}

	Future<void> _open(LibraryItem item) async {
		Navigator.of(context).push(MaterialPageRoute(
			builder: (_) => WorkPage(vault: widget.vault, details: item.work),
		));
		await _load();
	}

	@override
	Widget build(BuildContext context) {
		final shown = items;
		return Scaffold(
			appBar: AppBar(title: const Text('Library')),
			body: shown == null
					? const Center(child: CircularProgressIndicator())
					: RefreshIndicator(
						onRefresh: () async {
							await _load();
							SyncScheduler.instance.nudge();
						},
						child: CustomScrollView(
							physics: const AlwaysScrollableScrollPhysics(),
							slivers: [
								SliverToBoxAdapter(
									child: ContinueReadingRow(vault: widget.vault, onChanged: () {}),
								),
								if (shown.isEmpty)
									const SliverFillRemaining(
										hasScrollBody: false,
										child: Center(child: Text('Nothing in your library yet')),
									)
								else
									SliverList(
										delegate: SliverChildBuilderDelegate(
											(context, index) {
												final item = shown[index];
												final cover = item.work.coverUrl;
												return ListTile(
													leading: cover != null
															? Image.network(cover, width: 40)
															: const Icon(Icons.menu_book),
													title: Text(item.work.title),
													subtitle: Text('${item.work.chapters.length} chapters'),
													onTap: () => _open(item),
												);
											},
											childCount: shown.length,
										),
									),
							],
						),
					),
		);
	}
}
