import 'package:flutter/material.dart';

import '../service/vault_service.dart';
import 'work_page.dart';

class DiscoverPage extends StatefulWidget {
	const DiscoverPage({super.key, required this.vault});

	final VaultService vault;

	@override
	State<DiscoverPage> createState() => _DiscoverPageState();
}

class _DiscoverPageState extends State<DiscoverPage> {
	List<SourceSummary> sources = [];
	String? selectedSource;

	@override
	void initState() {
		super.initState();
		_loadSources();
	}

	Future<void> _loadSources() async {
		final loaded = await widget.vault.listSources();
		setState(() {
			sources = loaded;
			selectedSource ??= loaded.firstOrNull?.id;
		});
	}

	Future<void> _open(WorkSummary work) async {
		if (selectedSource == null) return;
		final details = await widget.vault.importWork(sourceId: selectedSource!, remoteUrl: work.remoteUrl);
		if (!mounted) return;
		Navigator.of(context).push(MaterialPageRoute(builder: (_) => WorkPage(vault: widget.vault, details: details)));
	}

	@override
	Widget build(BuildContext context) {
		return DefaultTabController(
			length: sources.length,
			child: Scaffold(
				appBar: AppBar(
					title: const Text('Manga Vault'),
					bottom: TabBar(
						tabs: [for (final source in sources) Tab(text: source.name)],
						onTap: (index) => setState(() => selectedSource = sources[index].id),
					),
				),
				body: sources.isEmpty
						? const Center(child: Text('No sources installed'))
						: SourceResults(vault: widget.vault, sourceId: selectedSource!, onOpen: _open),
			),
		);
	}
}

class SourceResults extends StatefulWidget {
	const SourceResults({super.key, required this.vault, required this.sourceId, required this.onOpen});

	final VaultService vault;
	final String sourceId;
	final ValueChanged<WorkSummary> onOpen;

	@override
	State<SourceResults> createState() => _SourceResultsState();
}

class _SourceResultsState extends State<SourceResults> {
	final query = TextEditingController();
	List<WorkSummary>? results;

	@override
	void didUpdateWidget(SourceResults oldWidget) {
		super.didUpdateWidget(oldWidget);
		results = null;
		_loadLatest();
	}

	@override
	void initState() {
		super.initState();
		_loadLatest();
	}

	Future<void> _loadLatest() async {
		try {
			final latest = await widget.vault.latestSource(sourceId: widget.sourceId, page: 1);
			if (!mounted) return;
			setState(() => results = latest);
		} catch (_) {
			setState(() => results = []);
		}
	}

	Future<void> _search() async {
		final found = await widget.vault.searchSource(sourceId: widget.sourceId, query: query.text, page: 1);
		setState(() => results = found);
	}

	@override
	Widget build(BuildContext context) {
		final shown = results;
		return Column(
			children: [
				Padding(
					padding: const EdgeInsets.all(12),
					child: TextField(
						controller: query,
						onSubmitted: (_) => _search(),
						decoration: InputDecoration(
							hintText: 'Search',
							suffixIcon: IconButton(icon: const Icon(Icons.search), onPressed: _search),
						),
					),
				),
				Expanded(
					child: shown == null
							? const Center(child: CircularProgressIndicator())
							: GridView.builder(
								padding: const EdgeInsets.all(16),
								gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
									maxCrossAxisExtent: 160,
									childAspectRatio: 2 / 3,
									crossAxisSpacing: 12,
									mainAxisSpacing: 12,
								),
								itemCount: shown.length,
								itemBuilder: (context, index) => WorkCard(work: shown[index], onOpen: () => widget.onOpen(shown[index])),
							),
				),
			],
		);
	}
}

class WorkCard extends StatelessWidget {
	const WorkCard({super.key, required this.work, required this.onOpen});

	final WorkSummary work;
	final VoidCallback onOpen;

	@override
	Widget build(BuildContext context) {
		return InkWell(
			onTap: onOpen,
			borderRadius: BorderRadius.circular(8),
			child: ClipRRect(
				borderRadius: BorderRadius.circular(8),
				child: Stack(
					fit: StackFit.expand,
					children: [
						work.coverUrl != null
								? Image.network(work.coverUrl!, fit: BoxFit.cover)
								: const ColoredBox(color: Color(0xFF322820), child: Icon(Icons.menu_book)),
						Positioned(
							left: 0,
							right: 0,
							bottom: 0,
							child: DecoratedBox(
								decoration: BoxDecoration(
									gradient: LinearGradient(
										begin: Alignment.topCenter,
										end: Alignment.bottomCenter,
										colors: [Colors.transparent, Colors.black.withValues(alpha: 0.85)],
									),
								),
								child: Padding(
									padding: const EdgeInsets.fromLTRB(8, 20, 8, 8),
									child: Text(
										work.title,
										maxLines: 2,
										overflow: TextOverflow.ellipsis,
										style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.white),
									),
								),
							),
						),
					],
				),
			),
		);
	}
}
