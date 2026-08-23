import 'package:flutter/material.dart';

import '../service/vault_service.dart';
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
					: shown.isEmpty
						? const Center(child: Text('Nothing in your library yet'))
						: ListView.builder(
							itemCount: shown.length,
							itemBuilder: (context, index) => ListTile(
								leading: shown[index].work.coverUrl != null
										? Image.network(shown[index].work.coverUrl!, width: 40)
										: const Icon(Icons.menu_book),
								title: Text(shown[index].work.title),
								subtitle: Text('${shown[index].work.chapters.length} chapters'),
								onTap: () => _open(shown[index]),
							),
						),
		);
	}
}
