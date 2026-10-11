import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:path_provider/path_provider.dart';

import '../service/app_prefs.dart';
import '../service/local_service.dart';
import '../service/remote_service.dart';
import '../service/session_store.dart';
import '../service/sync_engine.dart';
import '../service/vault_service.dart';
import 'appearance_section.dart';
import 'server_sign_in.dart';
import 'sessions_section.dart';
import 'stats.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({
    super.key,
    required this.service,
    required this.isLocal,
    required this.restartSession,
    this.linkedRemote,
  });

  final VaultService service;
  final bool isLocal;
  final VoidCallback restartSession;
  final RemoteService? linkedRemote;

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
  final sessions = GlobalKey<SessionsSectionState>();

  RemoteService? get _account {
    final linkedRemote = widget.linkedRemote;
    if (linkedRemote != null) return linkedRemote;
    final service = widget.service;
    return service is RemoteService ? service : null;
  }

  bool get _showTrackers => _account?.supportsTrackers ?? false;

  @override
  void initState() {
    super.initState();
    _restoreLink().whenComplete(_loadTrackers);
  }

  Future<void> _loadTrackers() async {
    final account = _account;
    if (account == null || !account.supportsTrackers) return;
    try {
      serverBaseUrl = account.baseUrl;
      trackerRegistry = [
        for (final entry in await account.trackersRegistry()) entry,
      ];
      linkedTrackerIds = (await account.myTrackerAccounts())
          .map((tracker) => tracker['tracker_id'] as String)
          .toSet();
    } catch (_) {
      trackerRegistry = const [];
    }
    if (mounted) setState(() {});
  }

  bool _trackerLinked(String id) => linkedTrackerIds.contains(id);

  Future<void> _linkCredentials(String id) async {
    final remote = _account;
    if (remote == null) return;
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
    final remote = _account;
    if (remote == null || serverBaseUrl == null) return;
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
    final remote = _account;
    if (remote == null) return;
    final accounts = await remote.myTrackerAccounts();
    setState(
      () => linkedTrackerIds = {
        for (final account in accounts) account['tracker_id'] as String,
      },
    );
  }

  Future<void> _linkTracker(String id) async {
    final remote = _account;
    if (remote == null) return;
    final controller = trackerTokenFields.putIfAbsent(
      id,
      TextEditingController.new,
    );
    final token = controller.text.trim();
    if (token.isEmpty) return;
    await remote.linkTracker(trackerId: id, token: token);
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text('$id linked')));
  }

  Future<void> _unlinkTracker(String id) async {
    final remote = _account;
    if (remote == null) return;
    await remote.unlinkTracker(trackerId: id);
    setState(() => linkedTrackerIds.remove(id));
  }

  Future<void> _restoreLink() async {
    try {
      final saved = await SessionStore.read();
      if (saved['kind'] != 'link') return;
      final baseUrl = saved['base_url'] as String?;
      final token = saved['token'] as String?;
      if (baseUrl == null || token == null) return;
      setState(() => linked = RemoteService(baseUrl: baseUrl, token: token));
    } catch (_) {}
  }

  Future<void> _link() async {
    final service = await showServerSignIn(
      context,
      title: 'Link to a server account',
      submitLabel: 'Link',
      failureLabel: 'Link failed',
    );
    if (service == null) return;
    await SessionStore.saveLink(baseUrl: service.baseUrl, token: service.token);
    widget.restartSession();
  }

  Future<void> _unlink() async {
    await SessionStore.saveLocal();
    widget.restartSession();
  }

  Future<void> _signOut() async {
    final service = widget.service;
    if (service is RemoteService) {
      try {
        await service.logout();
      } catch (_) {}
    }
    await SessionStore.saveLocal();
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
      body: NotificationListener<ScrollNotification>(
        onNotification: (notification) {
          sessions.currentState?.loadMore(notification);
          return false;
        },
        child: ListView(
          children: [
            const AppearanceSection(),
            const Padding(
              padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
              child: Text(
                'READING',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  letterSpacing: 0.6,
                ),
              ),
            ),
            ListTile(
              leading: const Icon(Icons.insights_outlined),
              title: const Text('Statistics'),
              subtitle: const Text('Chapters read, streak and daily activity.'),
              onTap: () => Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => StatsPage(vault: widget.service),
                ),
              ),
            ),
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
                subtitle: const Text(
                  'Library and progress are stored locally.',
                ),
              )
            else
              ListTile(
                title: const Text('Server account'),
                subtitle: const Text(
                  'Library and progress live on the server.',
                ),
                trailing: FilledButton(
                  onPressed: _signOut,
                  child: const Text('Sign out'),
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
                  linked == null
                      ? 'Not linked'
                      : 'Linked to ${linked!.baseUrl}',
                ),
                subtitle: const Text(
                  'Keep library and reading progress in step with a server account.',
                ),
                trailing: linked == null
                    ? FilledButton(onPressed: _link, child: const Text('Link'))
                    : TextButton(
                        onPressed: _unlink,
                        child: const Text('Unlink'),
                      ),
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
            if (_showTrackers)
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
            if (_showTrackers)
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
            if (_account != null)
              SessionsSection(
                key: sessions,
                account: _account!,
                onSignedOut: widget.restartSession,
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
                      : (value) async {
                          await prefs.setChapterNotifications(value);
                          if (mounted) setState(() {});
                        },
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
            const Padding(
              padding: EdgeInsets.fromLTRB(16, 24, 16, 4),
              child: Text(
                'DATA',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  letterSpacing: 0.6,
                ),
              ),
            ),
            ListTile(
              leading: const Icon(Icons.download),
              title: const Text('Export backup'),
              subtitle: const Text('Save all library data as a JSON file.'),
              onTap: () async {
                try {
                  final json = await widget.service.exportBackup();
                  final file = await _backupFile();
                  await file.writeAsString(json);
                  if (context.mounted)
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Saved to ${file.path}')),
                    );
                } catch (e) {
                  if (context.mounted)
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Export failed: $e')),
                    );
                }
              },
            ),
            ListTile(
              leading: const Icon(Icons.upload),
              title: const Text('Import backup'),
              subtitle: const Text('Restore library data from a JSON backup.'),
              onTap: () async {
                try {
                  final file = await _backupFile();
                  if (!await file.exists()) {
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('No backup at ${file.path}')),
                      );
                    }
                    return;
                  }
                  await widget.service.importBackup(await file.readAsString());
                  if (!context.mounted) return;
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Backup restored')),
                  );
                  widget.restartSession();
                } catch (e) {
                  if (context.mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Import failed: $e')),
                    );
                  }
                }
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<File> _backupFile() async {
    final dir = await getApplicationDocumentsDirectory();
    return File('${dir.path}/manga-vault-backup.json');
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
