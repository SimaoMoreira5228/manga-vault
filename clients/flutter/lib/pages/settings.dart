import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import '../service/local_service.dart';
import '../service/remote_service.dart';
import '../service/sync_engine.dart';
import 'dart:convert';
import 'dart:io';

class SettingsPage extends StatefulWidget {
	const SettingsPage({super.key, required this.service});

	final LocalService service;

	@override
	State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
	RemoteService? linked;
	bool syncing = false;

	@override
	void initState() {
		super.initState();
		_restoreLink();
	}

	Future<File> _linkFile() async {
		final support = await getApplicationSupportDirectory();
		return File('${support.path}/remote.json');
	}

	Future<void> _restoreLink() async {
		try {
			final file = await _linkFile();
			if (!file.existsSync()) return;
			final saved = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
			if (saved['kind'] != 'link') return;
			setState(() => linked = RemoteService(
				baseUrl: saved['base_url'] as String,
				token: saved['token'] as String,
			));
		} catch (_) {}
	}

	Future<void> _saveKind(Map<String, dynamic> payload) async {
		final file = await _linkFile();
		await file.writeAsString(jsonEncode(payload));
	}

	Future<void> _link() async {
		final baseUrl = TextEditingController();
		final username = TextEditingController();
		final password = TextEditingController();
		await showDialog<void>(
			context: context,
			builder: (context) => AlertDialog(
				title: const Text('Link to a server account'),
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
								final remote = await RemoteService.login(
									baseUrl: baseUrl.text.trim(),
									username: username.text,
									password: password.text,
								);
								await _saveKind({'kind': 'link', 'base_url': remote.baseUrl, 'token': remote.token});
								if (!context.mounted) return;
								Navigator.of(context).pop();
								setState(() => linked = remote);
							} catch (e) {
								if (!context.mounted) return;
								ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Link failed: $e')));
							}
						},
						child: const Text('Link'),
					),
				],
			),
		);
	}

	Future<void> _unlink() async {
		final file = await _linkFile();
		if (file.existsSync()) await file.delete();
		setState(() => linked = null);
	}

	Future<void> _syncNow() async {
		if (linked == null) return;
		setState(() => syncing = true);
		try {
			await SyncEngine(local: widget.service, remote: linked!).synchronize();
			if (!mounted) return;
			ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Sync complete')));
		} catch (e) {
			if (!mounted) return;
			ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Sync failed: $e')));
		} finally {
			if (mounted) setState(() => syncing = false);
		}
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			appBar: AppBar(title: const Text('Settings')),
			body: ListView(
				children: [
					const Padding(
						padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
						child:
							Text('SYNC', style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, letterSpacing: 0.6)),
					),
					ListTile(
						title: Text(linked == null ? 'Not linked' : 'Linked to ${linked!.baseUrl}'),
						subtitle: const Text('Sync library and reading progress with a server account'),
						trailing: linked == null
								? FilledButton(onPressed: _link, child: const Text('Link'))
								: TextButton(onPressed: _unlink, child: const Text('Unlink')),
					),
					if (linked != null)
						ListTile(
							title: const Text('Sync now'),
							trailing: syncing
									? const CircularProgressIndicator()
									: IconButton(icon: const Icon(Icons.sync), onPressed: _syncNow),
						),
					const Padding(
						padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
						child:
							Text('TRANSLATION', style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, letterSpacing: 0.6)),
					),
					FutureBuilder<String>(
						future: widget.service.translationMode(),
						builder: (context, snapshot) {
							final mode = snapshot.data ?? 'off';
							return ListTile(
								title: Text(switch (mode) {
									'byok' => 'Your own API key',
									'ollama' => 'Ollama endpoint',
									_ => 'Disabled',
								}),
								subtitle: const Text('Translate novels while reading. Stored on this device only.'),
								trailing: mode == 'off'
										? FilledButton(onPressed: _configureTranslation, child: const Text('Set up'))
										: Row(mainAxisSize: MainAxisSize.min, children: [
											TextButton(onPressed: _configureTranslation, child: const Text('Change')),
											TextButton(onPressed: () async {
												await widget.service.clearTranslationProvider();
												if (context.mounted) setState(() {});
											}, child: const Text('Clear')),
										]),
							);
						},
					),
				],
			),
		);
	}

	Future<void> _configureTranslation() async {
		await showDialog<void>(
			context: context,
			builder: (context) => _TranslationConfigDialog(service: widget.service),
		);
		if (mounted) setState(() {});
	}
}

class _TranslationConfigDialog extends StatefulWidget {
	const _TranslationConfigDialog({required this.service});

	final LocalService service;

	@override
	State<_TranslationConfigDialog> createState() => _TranslationConfigDialogState();
}

class _TranslationConfigDialogState extends State<_TranslationConfigDialog> {
	var useKey = false;
	final endpoint = TextEditingController();
	final key = TextEditingController();
	final model = TextEditingController();

	@override
	Widget build(BuildContext context) {
		return AlertDialog(
			title: const Text('Translation provider'),
			content: Column(
				mainAxisSize: MainAxisSize.min,
				children: [
					RadioGroup<bool>(
						groupValue: useKey,
						onChanged: (value) => setState(() => useKey = value!),
						child: Column(
							mainAxisSize: MainAxisSize.min,
							children: [
								RadioListTile<bool>(
									value: false,
									title: const Text('Ollama endpoint'),
									subtitle: const Text('e.g. http://localhost:11434'),
								),
								RadioListTile<bool>(
									value: true,
									title: const Text('API key (OpenAI-compatible)'),
									subtitle: const Text('Gemini, OpenRouter, OpenAI…'),
								),
							],
						),
					),
					if (!useKey)
						TextField(controller: endpoint, decoration: const InputDecoration(hintText: 'http://localhost:11434')),
					if (useKey) ...[
						TextField(controller: key, obscureText: true, decoration: const InputDecoration(hintText: 'API key')),
						TextField(controller: endpoint, decoration: const InputDecoration(hintText: 'Base URL (optional)')),
					],
					TextField(controller: model, decoration: const InputDecoration(hintText: 'Model (optional)')),
				],
			),
			actions: [
				TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
				FilledButton(
					onPressed: () async {
						await widget.service.setTranslationProvider(
							providerBaseUrl: endpoint.text.trim().isEmpty ? null : endpoint.text.trim(),
							apiKey: key.text.isEmpty ? null : key.text,
							model: model.text.trim().isEmpty ? null : model.text.trim(),
						);
						if (context.mounted) Navigator.of(context).pop();
					},
					child: const Text('Save'),
				),
			],
		);
	}
}
