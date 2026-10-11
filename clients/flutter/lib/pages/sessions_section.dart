import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/session_store.dart';
import '../service/vault_service.dart';

class SessionsSection extends StatefulWidget {
  const SessionsSection({
    super.key,
    required this.account,
    required this.onSignedOut,
  });

  final RemoteService account;
  final VoidCallback onSignedOut;

  @override
  State<SessionsSection> createState() => _SessionsSectionState();
}

class _SessionsSectionState extends State<SessionsSection> {
  List<SessionInfo> sessions = const [];
  bool loading = true;
  String? failure;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final loaded = await widget.account.sessions();
      if (!mounted) return;
      setState(() {
        sessions = loaded;
        loading = false;
        failure = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        loading = false;
        failure = '$e';
      });
    }
  }

  Future<void> _revoke(SessionInfo session) async {
    try {
      await widget.account.revokeSession(session.token);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Revoke failed: $e')));
      return;
    }
    if (session.token == widget.account.token) {
      await SessionStore.saveLocal();
      widget.onSignedOut();
      return;
    }
    await _load();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Padding(
          padding: EdgeInsets.fromLTRB(16, 16, 16, 4),
          child: Text(
            'LOGGED-IN DEVICES',
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w500,
              letterSpacing: 0.6,
            ),
          ),
        ),
        if (loading)
          const ListTile(title: Text('Loading devices…'))
        else if (failure != null)
          ListTile(
            title: Text('Could not load devices: $failure'),
            trailing: IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: _load,
            ),
          )
        else if (sessions.isEmpty)
          const ListTile(title: Text('No logged-in devices'))
        else
          for (final session in sessions)
            ListTile(
              title: Text(session.deviceLabel ?? 'Unnamed device'),
              subtitle: Text('Last seen ${_timestamp(session.lastSeenAt)}'),
              trailing: TextButton(
                onPressed: () => _revoke(session),
                style: TextButton.styleFrom(
                  foregroundColor: Theme.of(context).colorScheme.error,
                ),
                child: const Text('Revoke'),
              ),
            ),
      ],
    );
  }
}

String _timestamp(DateTime moment) {
  final local = moment.toLocal();
  return '${local.year}-${_pad(local.month)}-${_pad(local.day)}'
      ' ${_pad(local.hour)}:${_pad(local.minute)}';
}

String _pad(int value) => value.toString().padLeft(2, '0');
