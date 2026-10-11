import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/session_store.dart';
import '../service/vault_service.dart';
import 'pager_bar.dart';

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
  int total = 0;
  int offset = 0;
  bool loading = true;
  String? failure;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final page = await widget.account.sessions(offset: offset);
      if (!mounted) return;
      setState(() {
        sessions = page.items;
        total = page.total;
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

  void _shiftPage(int next) {
    setState(() => offset = next);
    _load();
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
    if (sessions.length == 1 && offset > 0) {
      setState(() => offset -= pageSize);
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
        if (failure == null)
          PagerBar(
            offset: offset,
            limit: pageSize,
            total: total,
            onshift: _shiftPage,
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
