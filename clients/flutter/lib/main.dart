import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import 'src/rust/api/local.dart' as local;
import 'src/rust/frb_generated.dart';

Future<void> main() async {
	WidgetsFlutterBinding.ensureInitialized();
	await RustLib.init();
	runApp(const MangaVaultApp());
}

class MangaVaultApp extends StatelessWidget {
	const MangaVaultApp({super.key});

	@override
	Widget build(BuildContext context) {
		return MaterialApp(
			title: 'Manga Vault',
			theme: ThemeData(colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo), useMaterial3: true),
			darkTheme: ThemeData(
				colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo, brightness: Brightness.dark),
				useMaterial3: true,
			),
			home: const DiscoverPage(),
		);
	}
}

class DiscoverPage extends StatefulWidget {
	const DiscoverPage({super.key});

	@override
	State<DiscoverPage> createState() => _DiscoverPageState();
}

class _DiscoverPageState extends State<DiscoverPage> {
	local.LocalVault? vault;
	List<local.SourceSummary> sources = [];
	String? error;

	@override
	void initState() {
		super.initState();
		_start();
	}

	Future<void> _start() async {
		try {
			final docs = await getApplicationSupportDirectory();
			final started = await local.start(
				dataDir: '${docs.path}/local',
				pluginsDir: '${docs.path}/plugins',
			);
			setState(() {
				vault = started;
				error = null;
			});
			await _loadSources();
		} catch (e) {
			setState(() => error = e.toString());
		}
	}

	Future<void> _loadSources() async {
		final loaded = await vault!.listSources();
		setState(() => sources = loaded);
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			appBar: AppBar(title: const Text('Manga Vault')),
			body: error != null
					? Center(child: Text(error!))
					: sources.isEmpty
						? const Center(child: CircularProgressIndicator())
						: ListView.builder(
							itemCount: sources.length,
							itemBuilder: (context, index) => ListTile(
								title: Text(sources[index].name),
								subtitle: Text('${sources[index].kind} · v${sources[index].version}'),
								onTap: () => Navigator.of(context).push(MaterialPageRoute(
									builder: (_) => SourcePage(vault: vault!, source: sources[index]),
								)),
							),
						),
		);
	}
}

class SourcePage extends StatefulWidget {
	const SourcePage({super.key, required this.vault, required this.source});

	final local.LocalVault vault;
	final local.SourceSummary source;

	@override
	State<SourcePage> createState() => _SourcePageState();
}

class _SourcePageState extends State<SourcePage> {
	final query = TextEditingController();
	List<local.WorkSummary> results = [];

	Future<void> _search() async {
		final found = await widget.vault.searchSource(
			sourceId: widget.source.id,
			query: query.text,
			page: 1,
		);
		setState(() => results = found);
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			appBar: AppBar(title: Text(widget.source.name)),
			body: Column(
				children: [
					Padding(
						padding: const EdgeInsets.all(12),
						child: TextField(
							controller: query,
							onSubmitted: (_) => _search(),
							decoration: InputDecoration(
								hintText: 'Search ${widget.source.name}',
								suffixIcon: IconButton(icon: const Icon(Icons.search), onPressed: _search),
							),
						),
					),
					Expanded(
						child: ListView.builder(
							itemCount: results.length,
							itemBuilder: (context, index) => ListTile(title: Text(results[index].title)),
						),
					),
				],
			),
		);
	}
}
