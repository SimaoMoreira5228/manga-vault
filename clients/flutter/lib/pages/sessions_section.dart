import 'package:flutter/material.dart';

import '../service/password_form.dart';
import '../service/remote_error.dart';
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
  bool securityBusy = false;
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
      await _signOutEverywhere();
      return;
    }
    if (sessions.length == 1 && offset > 0) {
      setState(() => offset -= pageSize);
    }
    await _load();
  }

  Future<void> _signOutEverywhere() async {
    await SessionStore.saveLocal();
    if (!mounted) return;
    widget.onSignedOut();
  }

  Future<void> _logoutEverywhere() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Log out everywhere?'),
        content: const Text(
          'Every device is signed out, including this one. You will sign in again.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Log out'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    setState(() => securityBusy = true);
    try {
      await widget.account.logoutEverywhere();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Sign out failed: $e')));
      }
    }
    if (mounted) setState(() => securityBusy = false);
    await _signOutEverywhere();
  }

  Future<void> _changePassword() async {
    final changed = await showDialog<int>(
      context: context,
      builder: (context) => ChangePasswordDialog(account: widget.account),
    );
    if (changed == null) return;
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(passwordChangeNotice(changed))));
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
        ListTile(
          title: const Text('Change password'),
          subtitle: const Text(
            'Signs out every other device; this one keeps working.',
          ),
          trailing: FilledButton(
            onPressed: securityBusy ? null : _changePassword,
            child: const Text('Change'),
          ),
        ),
        ListTile(
          title: const Text('Log out everywhere'),
          subtitle: const Text('Signs out this device too.'),
          trailing: TextButton(
            onPressed: securityBusy ? null : _logoutEverywhere,
            style: TextButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Log out everywhere'),
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

class ChangePasswordDialog extends StatefulWidget {
  const ChangePasswordDialog({super.key, required this.account});

  final RemoteService account;

  @override
  State<ChangePasswordDialog> createState() => _ChangePasswordDialogState();
}

class _ChangePasswordDialogState extends State<ChangePasswordDialog> {
  final current = TextEditingController();
  final next = TextEditingController();
  final repeat = TextEditingController();
  String? field;
  String? problem;
  bool busy = false;

  @override
  void dispose() {
    current.dispose();
    next.dispose();
    repeat.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final failure = checkPasswordChange(
      current: current.text,
      next: next.text,
      repeat: repeat.text,
    );
    if (failure != null) {
      setState(() {
        field = failure.field;
        problem = failure.message;
      });
      return;
    }
    setState(() {
      busy = true;
      field = null;
      problem = null;
    });
    try {
      final revoked = await widget.account.changePassword(
        currentPassword: current.text,
        newPassword: next.text,
      );
      if (mounted) Navigator.of(context).pop(revoked);
    } on RemoteApiException catch (e) {
      if (!mounted) return;
      setState(() {
        busy = false;
        field = e.code == 'wrong_password' ? 'current' : null;
        problem = e.code == 'wrong_password'
            ? 'That is not your current password.'
            : e.code == 'too_many_attempts' || e.code == 'too_many_requests'
            ? e.retryAfterSeconds == null
                  ? e.message
                  : 'Too many attempts. Retry in ${e.retryAfterSeconds}s.'
            : e.message;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Change password'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: current,
            obscureText: true,
            autofillHints: const [AutofillHints.password],
            decoration: InputDecoration(
              labelText: 'Current password',
              errorText: field == 'current' ? problem : null,
            ),
          ),
          TextField(
            controller: next,
            obscureText: true,
            decoration: InputDecoration(
              labelText: 'New password',
              helperText: 'At least $minPasswordLength characters.',
              errorText: field == 'next' ? problem : null,
            ),
          ),
          TextField(
            controller: repeat,
            obscureText: true,
            decoration: InputDecoration(
              labelText: 'Repeat new password',
              errorText: field == 'confirm' ? problem : null,
            ),
          ),
          if (field == null && problem != null)
            Text(problem!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
        ],
      ),
      actions: [
        TextButton(
          onPressed: busy ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: busy ? null : _submit,
          child: Text(busy ? 'Changing…' : 'Change password'),
        ),
      ],
    );
  }
}
