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
import 'service/vault_service.dart';
import 'src/rust/api/local.dart' as local;
import 'src/rust/frb_generated.dart';
import 'theme.dart';

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
			theme: buildVaultTheme(),
			home: const ConnectPage(),
		);
	}
}

File _remoteStateFile(Directory support) => File('${support.path}/remote.json');

class ConnectPage extends StatefulWidget {
	const ConnectPage({super.key});

	@override
	State<ConnectPage> createState() => _ConnectPageState();
}

class _ConnectPageState extends State<ConnectPage> {
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
			final support = await getApplicationSupportDirectory();
			final file = _remoteStateFile(support);
			if (!file.existsSync()) {
				setState(() => busy = false);
				return;
			}
			final saved = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
			final kind = saved['kind'] ?? 'remote';
			if (kind == 'link') {
				final vault = await local.start(
					dataDir: '${support.path}/local',
					pluginsDir: '${support.path}/plugins',
				);
				final service = LocalService(vault);
				final remote = RemoteService(baseUrl: saved['base_url'] as String, token: saved['token'] as String);
				await _enter(service, true, remote: remote);
				return;
			}
			await _enter(RemoteService(baseUrl: saved['base_url'] as String, token: saved['token'] as String), false);
		} catch (_) {
			setState(() => busy = false);
		}
	}

	Future<void> _enter(VaultService service, bool isLocal, {RemoteService? remote}) async {
		if (!mounted) return;
		Navigator.of(context).pushReplacement(MaterialPageRoute(
			builder: (_) => HomePage(service: service, isLocal: isLocal),
		));
		if (remote != null) {
			unawaited(SyncEngine(local: service, remote: remote).synchronize().then<void>((_) {}, onError: (_) {}));
		}
	}

	Future<void> _startLocal() async {
		setState(() {
			busy = true;
			error = null;
		});
		try {
			final support = await getApplicationSupportDirectory();
			final vault = await local.start(
				dataDir: '${support.path}/local',
				pluginsDir: '${support.path}/plugins',
			);
			final profiles = await vault.profiles();
			final unlocked = profiles.length == 1 && !profiles.first.hasPin;
			if (!mounted) return;
			if (unlocked) {
				await vault.selectProfile(id: profiles.first.id);
				await _enter(LocalService(vault), true);
			} else {
				setState(() {
					busy = false;
					pendingVault = vault;
					pendingProfiles = profiles;
				});
			}
		} catch (e) {
			setState(() {
				busy = false;
				error = e.toString();
			});
		}
	}

	@override
	Widget build(BuildContext context) {
		if (busy) return const Scaffold(body: Center(child: CircularProgressIndicator()));
		if (pendingVault != null) {
			return ProfilePickerPage(
				vault: pendingVault!,
				profiles: pendingProfiles,
				onSelected: (service) => _enter(service, true),
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
							FilledButton(onPressed: _startLocal, child: const Text('Use this device')),
							const SizedBox(height: 12),
							OutlinedButton(onPressed: () => _openRemoteForm(), child: const Text('Connect to a server')),
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

	Future<void> _openRemoteForm() async {
		final baseUrl = TextEditingController();
		final username = TextEditingController();
		final password = TextEditingController();
		final formKey = GlobalKey<FormState>();
		await showDialog<void>(
			context: context,
			builder: (context) => AlertDialog(
				title: const Text('Connect to a server'),
				content: Form(
					key: formKey,
					child: Column(
						mainAxisSize: MainAxisSize.min,
						children: [
							TextFormField(
								controller: baseUrl,
								decoration: const InputDecoration(hintText: 'https://server.example.org'),
								validator: (value) => (value == null || value.trim().isEmpty) ? 'Required' : null,
							),
							TextFormField(
								controller: username,
								decoration: const InputDecoration(hintText: 'Username'),
								validator: (value) => (value == null || value.isEmpty) ? 'Required' : null,
							),
							TextFormField(
								controller: password,
								obscureText: true,
								decoration: const InputDecoration(hintText: 'Password'),
								validator: (value) => (value == null || value.isEmpty) ? 'Required' : null,
							),
						],
					),
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
					FilledButton(
						onPressed: () async {
							if (!(formKey.currentState?.validate() ?? false)) return;
							try {
								final service = await RemoteService.login(
									baseUrl: baseUrl.text.trim(),
									username: username.text,
									password: password.text,
								);
								final support = await getApplicationSupportDirectory();
								_remoteStateFile(support).writeAsStringSync(
									jsonEncode({'kind': 'remote', 'base_url': service.baseUrl, 'token': service.token}),
								);
								if (!context.mounted) return;
								Navigator.of(context).pop();
								await _enter(service, false);
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
}

class HomePage extends StatelessWidget {
	const HomePage({super.key, required this.service, required this.isLocal});

	final VaultService service;
	final bool isLocal;

	@override
	Widget build(BuildContext context) {
		return DefaultTabController(
			length: isLocal ? 4 : 3,
			animationDuration: Duration.zero,
			child: Scaffold(
				body: TabBarView(
					children: [
						DiscoverPage(vault: service),
						LibraryPage(vault: service),
						SourcesPage(vault: service),
						if (isLocal) SettingsPage(service: service as LocalService),
					],
				),
				bottomNavigationBar: TabBar(
					tabs: [
						const Tab(icon: Icon(Icons.explore_outlined), text: 'Discover'),
						const Tab(icon: Icon(Icons.collections_bookmark_outlined), text: 'Library'),
						const Tab(icon: Icon(Icons.extension_outlined), text: 'Plugins'),
						if (isLocal) const Tab(icon: Icon(Icons.settings_outlined), text: 'Settings'),
					],
					labelColor: Theme.of(context).colorScheme.primary,
					unselectedLabelColor: Theme.of(context).colorScheme.onSurfaceVariant,
					indicatorColor: Theme.of(context).colorScheme.primary,
					dividerColor: Colors.transparent,
					labelStyle: const TextStyle(fontFamily: 'Geist', fontSize: 12, fontWeight: FontWeight.w500),
					unselectedLabelStyle: const TextStyle(fontFamily: 'Geist', fontSize: 12, fontWeight: FontWeight.w500),
				),
			),
		);
	}
}
