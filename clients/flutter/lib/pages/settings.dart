import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:path_provider/path_provider.dart';

import '../service/app_prefs.dart';
import '../service/local_service.dart';
import '../service/remote_service.dart';
import '../service/sync_engine.dart';
import '../service/vault_service.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({
    super.key,
    required this.service,
    required this.isLocal,
    required this.restartSession,
  });

  final VaultService service;
  final bool isLocal;
  final VoidCallback restartSession;

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  RemoteService? linked;
  bool syncing = false;
  List<Map<String, dynamic>> trackerRegistry = const [];
  Set<String> linkedTrackerIds = {};
  Map<String, TextEditingController> trackerTokenFields = {};
  Map<String, TextEditingController> trackerUsernameFields = {};
  Map<String, TextEditingController> trackerPasswordFields = {};
  String? serverBaseUrl;

  @override
  void initState() {
    super.initState();
    _restoreLink().whenComplete(_loadTrackers);
  }

  Future<File> _linkFile() async {
    final support = await getApplicationSupportDirectory();
    return File('${support.path}/remote.json');
  }

  Future<void> _loadTrackers() async {
    if (!widget.isLocal && widget.service is RemoteService) {
      final service = widget.service as RemoteService;
      try {
        serverBaseUrl = service.baseUrl;
        trackerRegistry = [
          for (final entry in await service.trackersRegistry()) entry,
        ];
        linkedTrackerIds = (await service.myTrackerAccounts())
            .map((account) => account['tracker_id'] as String)
            .toSet();
      } catch (_) {
        trackerRegistry = const [];
      }
      if (mounted) setState(() {});
    }
  }

  bool _trackerLinked(String id) => linkedTrackerIds.contains(id);

  Future<void> _linkCredentials(String id) async {
    final remote = widget.service;
    if (remote is! RemoteService) return;
    final username = trackerUsernameFields
        .putIfAbsent(id, TextEditingController.new)
        .text
        .trim();
    final password = trackerPasswordFields
        .putIfAbsent(id, TextEditingController.new)
        .text;
    if (username.isEmpty || password.isEmpty) return;
    await remote.linkTracker(
      trackerId: id,
      username: username,
      password: password,
    );
    await _reloadTrackers();
  }

  Future<void> _connectOauth(String id) async {
    final remote = widget.service;
    if (remote is! RemoteService || serverBaseUrl == null) return;
    final redirectUri = '${serverBaseUrl!}/api/me/trackers/$id/oauth/callback';
    final authorizeUrl = await remote.startTrackerOauth(
      trackerId: id,
      redirectUri: redirectUri,
    );
    final launched = await launchUrl(
      Uri.parse(authorizeUrl),
      mode: LaunchMode.externalApplication,
    );
    if (!launched) return;
    for (var attempt = 0; attempt < 40; attempt++) {
      await Future<void>.delayed(const Duration(seconds: 3));
      try {
        final accounts = await remote.myTrackerAccounts();
        if (accounts.any((account) => account['tracker_id'] == id)) break;
      } catch (_) {}
    }
    await _loadTrackers();
  }

  Future<void> _reloadTrackers() async {
    final remote = widget.service;
    if (remote is! RemoteService) return;
    final accounts = await remote.myTrackerAccounts();
    setState(
      () => linkedTrackerIds = {
        for (final account in accounts) account['tracker_id'] as String,
      },
    );
  }

  Future<void> _linkTracker(String id) async {
    final controller = trackerTokenFields.putIfAbsent(
      id,
      TextEditingController.new,
    );
    final token = controller.text.trim();
    if (token.isEmpty || widget.service is! RemoteService) return;
    final remote = widget.service as RemoteService;
    await remote.linkTracker(trackerId: id, token: token);
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text('$id linked')));
  }

  Future<void> _unlinkTracker(String id) async {
    if (widget.service is! RemoteService) return;
    final remote = widget.service as RemoteService;
    await remote.unlinkTracker(trackerId: id);
    setState(() => linkedTrackerIds.remove(id));
  }

  Future<void> _restoreLink() async {
    try {
      final file = await _linkFile();
      if (!file.existsSync()) return;
      final saved = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      if (saved['kind'] != 'link') return;
      setState(
        () => linked = RemoteService(
          baseUrl: saved['base_url'] as String,
          token: saved['token'] as String,
        ),
      );
    } catch (_) {}
  }

  Future<void> _saveKind(String kind) async {
    final file = await _linkFile();
    final existing = file.existsSync()
        ? jsonDecode(file.readAsStringSync()) as Map<String, dynamic>
        : <String, dynamic>{};
    existing['kind'] = kind;
    await file.writeAsString(jsonEncode(existing));
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
            TextField(
              controller: baseUrl,
              decoration: const InputDecoration(
                hintText: 'https://server.example.org',
              ),
            ),
            TextField(
              controller: username,
              decoration: const InputDecoration(hintText: 'Username'),
            ),
            TextField(
              controller: password,
              obscureText: true,
              decoration: const InputDecoration(hintText: 'Password'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () async {
              try {
                await RemoteService.login(
                  baseUrl: baseUrl.text.trim(),
                  username: username.text,
                  password: password.text,
                );
                await _saveKind('link');
                if (!context.mounted) return;
                Navigator.of(context).pop();
                widget.restartSession();
              } catch (e) {
                if (!context.mounted) return;
                ScaffoldMessenger.of(context)
                    .showSnackBar(SnackBar(content: Text('Link failed: $e')));
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
    if (file.existsSync()) {
      final saved = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      saved['kind'] = 'local';
      await file.writeAsString(jsonEncode(saved));
    }
    widget.restartSession();
  }

  Future<void> _switchToLocalDevice() async {
    await _saveKind('local');
    widget.restartSession();
  }

  Future<void> _syncNow() async {
    if (linked == null || syncing) return;
    setState(() => syncing = true);
    try {
      await SyncEngine(local: widget.service, remote: linked!).synchronize();
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(const SnackBar(content: Text('Sync complete')));
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Sync failed: $e')));
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
            child: Text(
              'CONNECTION',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w500,
                letterSpacing: 0.6,
              ),
            ),
          ),
          if (widget.isLocal)
            ListTile(
              title: const Text('This device'),
              subtitle: const Text('Library and progress are stored locally.'),
            )
          else
            ListTile(
              title: const Text('Server account'),
              subtitle: const Text('Library and progress live on the server.'),
              trailing: FilledButton(
                onPressed: _switchToLocalDevice,
                child: const Text('Use this device'),
              ),
            ),
          if (widget.isLocal) ...[
            const Padding(
              padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
              child: Text(
                'SYNC',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  letterSpacing: 0.6,
                ),
              ),
            ),
            ListTile(
              title: Text(
                linked == null ? 'Not linked' : 'Linked to ${linked!.baseUrl}',
              ),
              subtitle: const Text(
                'Keep library and reading progress in step with a server account.',
              ),
              trailing: linked == null
                  ? FilledButton(onPressed: _link, child: const Text('Link'))
                  : TextButton(onPressed: _unlink, child: const Text('Unlink')),
            ),
            if (linked != null)
              ListTile(
                title: const Text('Sync now'),
                subtitle: const Text(
                  'Also happens automatically in the background.',
                ),
                trailing: syncing
                    ? const CircularProgressIndicator()
                    : IconButton(
                        icon: const Icon(Icons.sync),
                        onPressed: _syncNow,
                      ),
              ),
          ],
          if (!widget.isLocal)
            const Padding(
              padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
              child: Text(
                'TRACKERS',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  letterSpacing: 0.6,
                ),
              ),
            ),
          if (!widget.isLocal)
            for (final tracker in trackerRegistry)
              ListTile(
                title: Text(
                  _trackerLinked(tracker['id'] as String)
                      ? '${tracker['id']} (linked)'
                      : tracker['id'] as String,
                ),
                trailing: _trackerLinked(tracker['id'] as String)
                    ? TextButton(
                        onPressed: () =>
                            _unlinkTracker(tracker['id'] as String),
                        child: const Text('Unlink'),
                      )
                    : null,
                subtitle: _trackerLinked(tracker['id'] as String)
                    ? null
                    : switch (tracker['auth'] as String?) {
                        'oauth' => TextButton(
                          onPressed: () =>
                              _connectOauth(tracker['id'] as String),
                          child: const Text('Connect in browser'),
                        ),
                        'credentials' => Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            SizedBox(
                              width: 130,
                              child: TextField(
                                controller: trackerUsernameFields.putIfAbsent(
                                  tracker['id'] as String,
                                  TextEditingController.new,
                                ),
                                decoration: const InputDecoration(
                                  hintText: 'username',
                                ),
                              ),
                            ),
                            const SizedBox(width: 8),
                            SizedBox(
                              width: 130,
                              child: TextField(
                                controller: trackerPasswordFields.putIfAbsent(
                                  tracker['id'] as String,
                                  TextEditingController.new,
                                ),
                                obscureText: true,
                                decoration: const InputDecoration(
                                  hintText: 'password',
                                ),
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.link),
                              onPressed: () =>
                                  _linkCredentials(tracker['id'] as String),
                            ),
                          ],
                        ),
                        _ => Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            SizedBox(
                              width: 200,
                              child: TextField(
                                controller: trackerTokenFields.putIfAbsent(
                                  tracker['id'] as String,
                                  TextEditingController.new,
                                ),
                                obscureText: true,
                                decoration: const InputDecoration(
                                  hintText: 'access token',
                                ),
                              ),
                            ),
                            IconButton(
                              icon: const Icon(Icons.link),
                              onPressed: () =>
                                  _linkTracker(tracker['id'] as String),
                            ),
                          ],
                        ),
                      },
              ),
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
            child: Text(
              'NOTIFICATIONS',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w500,
                letterSpacing: 0.6,
              ),
            ),
          ),
          FutureBuilder<AppPrefs>(
            future: AppPrefs.instance(),
            builder: (context, snapshot) {
              final prefs = snapshot.data;
              return SwitchListTile(
                title: const Text('New chapter alerts'),
                subtitle: const Text(
                  'Show a banner when library works get new chapters.',
                ),
                value: prefs?.chapterNotifications ?? false,
                onChanged: prefs == null
                    ? null
                    : (value) => prefs.setChapterNotifications(value),
              );
            },
          ),
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
            child: Text(
              'TRANSLATION',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w500,
                letterSpacing: 0.6,
              ),
            ),
          ),
          FutureBuilder<String>(
            future: widget.service.translationMode(),
            builder: (context, snapshot) {
              final mode = snapshot.data ?? 'off';
              return ListTile(
                title: Text(switch (mode) {
                  'byok' || 'instance' => 'Your own API key',
                  'ollama' => 'Ollama endpoint',
                  _ => 'Disabled',
                }),
                subtitle: Text(
                  widget.isLocal
                      ? 'Translate novels while reading. Stored on this device only.'
                      : 'Managed by the server account settings.',
                ),
                trailing: widget.isLocal
                    ? (mode == 'off'
                          ? FilledButton(
                              onPressed: _configureTranslation,
                              child: const Text('Set up'),
                            )
                          : Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                TextButton(
                                  onPressed: _configureTranslation,
                                  child: const Text('Change'),
                                ),
                                TextButton(
                                  onPressed: () async {
                                    await widget.service
                                        .clearTranslationProvider();
                                    if (context.mounted) setState(() {});
                                  },
                                  child: const Text('Clear'),
                                ),
                              ],
                            ))
                    : null,
              );
            },
          ),
        ],
      ),
    );
  }

  Future<void> _configureTranslation() async {
    if (widget.service is! LocalService) return;
    final service = widget.service as LocalService;
    await showDialog<void>(
      context: context,
      builder: (context) => _TranslationConfigDialog(service: service),
    );
    if (mounted) setState(() {});
  }
}

class _TranslationConfigDialog extends StatefulWidget {
  const _TranslationConfigDialog({required this.service});

  final LocalService service;

  @override
  State<_TranslationConfigDialog> createState() =>
      _TranslationConfigDialogState();
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
            TextField(
              controller: endpoint,
              decoration: const InputDecoration(
                hintText: 'http://localhost:11434',
              ),
            ),
          if (useKey) ...[
            TextField(
              controller: key,
              obscureText: true,
              decoration: const InputDecoration(hintText: 'API key'),
            ),
            TextField(
              controller: endpoint,
              decoration: const InputDecoration(
                hintText: 'Base URL (optional)',
              ),
            ),
          ],
          TextField(
            controller: model,
            decoration: const InputDecoration(hintText: 'Model (optional)'),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: () async {
            await widget.service.setTranslationProvider(
              providerBaseUrl: endpoint.text.trim().isEmpty
                  ? null
                  : endpoint.text.trim(),
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
