import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import 'pages/discover.dart';
import 'pages/library.dart';
import 'pages/profile_picker.dart';
import 'pages/settings.dart';
import 'pages/sources.dart';
import 'service/local_service.dart';
import 'service/remote_service.dart';
import 'service/sync_engine.dart';
import 'service/sync_scheduler.dart';
import 'service/vault_service.dart';
import 'src/rust/api/local.dart' as local;
import 'src/rust/frb_generated.dart';
import 'theme.dart';

Future<void> main() async {
	WidgetsFlutterBinding.ensureInitialized();
	await RustLib.init();
	runApp(const MangaVaultApp());
}

class MangaVaultApp extends StatefulWidget {
	const MangaVaultApp({super.key});

	@override
	State<MangaVaultApp> createState() => _MangaVaultAppState();
}

class _MangaVaultAppState extends State<MangaVaultApp> {
	int sessionEpoch = 0;

	@override
	Widget build(BuildContext context) {
		return MaterialApp(
			title: 'Manga Vault',
			theme: buildVaultTheme(),
			home: BootstrapPage(key: ValueKey(sessionEpoch), onSessionEnded: () => setState(() => sessionEpoch += 1)),
		);
	}
}

Future<File> _stateFile() async {
	final support = await getApplicationSupportDirectory();
	return File('${support.path}/remote.json');
}

typedef RestartSession = void Function();

class BootstrapPage extends StatefulWidget {
	const BootstrapPage({super.key, required this.onSessionEnded});

	final RestartSession onSessionEnded;

	@override
	State<BootstrapPage> createState() => _BootstrapPageState();
}

class _BootstrapPageState extends State<BootstrapPage> {
	bool busy = true;
	String? error;
	local.LocalVault? pendingVault;
	List<local.ProfileSummary> pendingProfiles = const [];

	@override
	void initState() {
		super.initState();
		_restore();
	}

	Future<void> _restore() async {
		try {
			final file = await _stateFile();
			if (!file.existsSync()) {
				setState(() => busy = false);
				return;
			}
			final saved = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
			switch (saved['kind']) {
				case 'local':
					await _enterLocal();
				case 'link':
					await _startLinked(saved);
				default:
					await _enterRemote(RemoteService(baseUrl: saved['base_url'] as String, token: saved['token'] as String));
			}
		} catch (_) {
			setState(() => busy = false);
		}
	}

	Future<(local.LocalVault, Directory)> _startLocalCore(Directory support) async {
		final vault = await local.start(
			dataDir: '${support.path}/local',
			pluginsDir: '${support.path}/plugins',
		);
		return (vault, support);
	}

	Future<void> _enterLocal() async {
		final support = await getApplicationSupportDirectory();
		final (vault, _) = await _startLocalCore(support);
		final profiles = await vault.profiles();
		if (profiles.length == 1 && !profiles.first.hasPin) {
			await vault.selectProfile(id: profiles.first.id);
			await _enter(LocalService(vault), isLocal: true);
			return;
		}
		setState(() {
			busy = false;
			pendingVault = vault;
			pendingProfiles = profiles;
		});
	}

	Future<void> _enterLocalAs(local.LocalVault vault, local.ProfileSummary profile) async {
		await vault.selectProfile(id: profile.id, pin: null);
		await _enter(LocalService(vault), isLocal: true);
	}

	Future<void> _startLinked(Map<String, dynamic> saved) async {
		final support = await getApplicationSupportDirectory();
		final (vault, _) = await _startLocalCore(support);
		final service = LocalService(vault);
		final remote = RemoteService(baseUrl: saved['base_url'] as String, token: saved['token'] as String);
		await _enter(service, isLocal: true, remote: remote);
	}

	Future<void> _enterRemote(RemoteService service) => _enter(service, isLocal: false);

	Future<void> _enter(VaultService service, {required bool isLocal, RemoteService? remote}) async {
		if (!mounted) return;
		Navigator.of(context).pushReplacement(MaterialPageRoute(
			builder: (_) => HomePage(
				service: service,
				isLocal: isLocal,
				restartSession: widget.onSessionEnded,
			),
		));
	}

	Future<void> _saveKind(String kind) async {
		final file = await _stateFile();
		final existing = file.existsSync()
				? jsonDecode(file.readAsStringSync()) as Map<String, dynamic>
				: <String, dynamic>{};
		existing['kind'] = kind;
		await file.writeAsString(jsonEncode(existing));
	}

	Future<void> _startFreshLocal() async {
		setState(() {
			busy = true;
			error = null;
		});
		try {
			await _saveKind('local');
			await _enterLocal();
		} catch (e) {
			setState(() {
				busy = false;
				error = e.toString();
			});
		}
	}

	Future<void> _connectServer() async {
		final baseUrl = TextEditingController();
		final username = TextEditingController();
		final password = TextEditingController();
		await showDialog<void>(
			context: context,
			builder: (context) => AlertDialog(
				title: const Text('Use a server account'),
				content: Column(
					mainAxisSize: MainAxisSize.min,
					children: [
						TextField(controller: baseUrl, decoration: const InputDecoration(hintText: 'https://server.example.org')),
						TextField(controller: username, decoration: const InputDecoration(hintText: 'Username')),
						TextField(controller: password, obscureText: true, decoration: const InputDecoration(hintText: 'Password')),
					],
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
					FilledButton(
						onPressed: () async {
							try {
								final service = await RemoteService.login(
									baseUrl: baseUrl.text.trim(),
									username: username.text,
									password: password.text,
								);
								await _saveKind('remote');
								if (!context.mounted) return;
								Navigator.of(context).pop();
								await _enterRemote(service);
							} catch (e) {
								if (!context.mounted) return;
								ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Login failed: $e')));
							}
						},
						child: const Text('Sign in'),
					),
				],
			),
		);
	}

	@override
	Widget build(BuildContext context) {
		if (busy) return const Scaffold(body: Center(child: CircularProgressIndicator()));
		if (pendingVault != null) {
			return ProfilePickerPage(
				vault: pendingVault!,
				profiles: pendingProfiles,
				onSelected: (profile) async {
					final vault = pendingVault!;
					await _saveKind('local');
					SyncScheduler.instance.stop();
					await _enterLocalAs(vault, profile);
				},
			);
		}
		return Scaffold(
			body: Center(
				child: ConstrainedBox(
					constraints: const BoxConstraints(maxWidth: 360),
					child: Column(
						mainAxisAlignment: MainAxisAlignment.center,
						crossAxisAlignment: CrossAxisAlignment.stretch,
						children: [
							Text('Manga Vault', style: Theme.of(context).textTheme.displayMedium, textAlign: TextAlign.center),
							const SizedBox(height: 8),
							Text('Private Archive', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
							const SizedBox(height: 48),
							FilledButton(onPressed: _startFreshLocal, child: const Text('Use this device')),
							const SizedBox(height: 12),
							OutlinedButton(onPressed: _connectServer, child: const Text('Use a server account')),
							if (error != null) ...[
								const SizedBox(height: 24),
								Text(error!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
							],
						],
					),
				),
			),
		);
	}
}

class HomePage extends StatefulWidget {
	const HomePage({
		super.key,
		required this.service,
		required this.isLocal,
		required this.restartSession,
		this.linkedRemote,
	});

	final VaultService service;
	final bool isLocal;
	final RestartSession restartSession;
	final RemoteService? linkedRemote;

	@override
	State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> with WidgetsBindingObserver {
	var tab = 0;

	@override
	void initState() {
		super.initState();
		WidgetsBinding.instance.addObserver(this);
		if (widget.isLocal && widget.linkedRemote != null) {
			final remote = widget.linkedRemote!;
			SyncScheduler.instance.start(() => SyncEngine(local: widget.service, remote: remote).synchronize());
		} else {
			SyncScheduler.instance.stop();
		}
	}

	@override
	void dispose() {
		WidgetsBinding.instance.removeObserver(this);
		super.dispose();
	}

	@override
	void didChangeAppLifecycleState(AppLifecycleState state) {
		if (state == AppLifecycleState.resumed) SyncScheduler.instance.run();
	}

	@override
	Widget build(BuildContext context) {
		final destinations = [
			const NavigationSection(icon: Icons.explore_outlined, label: 'Discover'),
			const NavigationSection(icon: Icons.collections_bookmark_outlined, label: 'Library'),
			const NavigationSection(icon: Icons.extension_outlined, label: 'Plugins'),
			if (widget.isLocal) const NavigationSection(icon: Icons.settings_outlined, label: 'Settings'),
		];
		return Scaffold(
			body: switch (tab) {
				0 => DiscoverPage(vault: widget.service),
				1 => LibraryPage(vault: widget.service),
				2 => SourcesPage(vault: widget.service),
				_ => SettingsPage(
					service: widget.service as LocalService,
					isLocal: widget.isLocal,
					restartSession: widget.restartSession,
				),
			},
			bottomNavigationBar: NavigationBar(
				selectedIndex: tab,
				onDestinationSelected: (index) => setState(() => tab = index),
				destinations: [
					for (final section in destinations)
						NavigationDestination(icon: Icon(section.icon), label: section.label),
				],
			),
		);
	}
}

class NavigationSection {
	const NavigationSection({required this.icon, required this.label});

	final IconData icon;
	final String label;
}
