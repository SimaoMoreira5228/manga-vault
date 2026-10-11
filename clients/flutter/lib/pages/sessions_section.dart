import 'package:flutter/material.dart';

import '../service/password_form.dart';
import '../service/remote_error.dart';
import '../service/remote_service.dart';
import '../service/session_store.dart';
import '../service/vault_service.dart';
import 'list_footer.dart';

class SessionsSection extends StatefulWidget {
  const SessionsSection({
    super.key,
    required this.account,
    required this.onSignedOut,
  });

  final RemoteService account;
  final VoidCallback onSignedOut;

  @override
  State<SessionsSection> createState() => SessionsSectionState();
}

class SessionsSectionState extends State<SessionsSection> {
  List<SessionInfo> sessions = const [];
  bool loading = true;
  bool loadingMore = false;
  bool done = false;
  bool securityBusy = false;
  String? failure;
  String? pageFailure;
  final devices = GlobalKey();
  int _epoch = 0;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final epoch = ++_epoch;
    try {
      final page = await widget.account.sessions(offset: 0);
      if (!mounted || epoch != _epoch) return;
      setState(() {
        sessions = page.items;
        loading = false;
        failure = null;
        pageFailure = null;
        loadingMore = false;
        done =
            page.items.length < page.limit || page.items.length >= page.total;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        loading = false;
        failure = '$e';
      });
    }
  }

  Future<void> _more() async {
    if (loading || loadingMore || done || failure != null) return;
    if (sessions.isEmpty) return;
    final epoch = _epoch;
    setState(() => loadingMore = true);
    try {
      final page = await widget.account.sessions(offset: sessions.length);
      if (!mounted || epoch != _epoch) return;
      final seen = sessions.length + page.items.length;
      setState(() {
        sessions = [...sessions, ...page.items];
        pageFailure = null;
        loadingMore = false;
        done =
            page.items.isEmpty ||
            page.items.length < page.limit ||
            seen >= page.total;
      });
    } catch (e) {
      if (!mounted || epoch != _epoch) return;
      setState(() {
        pageFailure = '$e';
        loadingMore = false;
      });
    }
  }

  void loadMore(ScrollNotification notification) {
    if (notification is! ScrollUpdateNotification) return;
    if (notification.metrics.axis != Axis.vertical) return;
    final viewport = notification.context?.findRenderObject();
    final section = devices.currentContext?.findRenderObject();
    if (viewport is! RenderBox || section is! RenderBox) return;
    if (!viewport.hasSize || !section.hasSize) return;
    final reach =
        section.localToGlobal(Offset.zero, ancestor: viewport).dy +
        section.size.height;
    if (reach > viewport.size.height + 200) return;
    _more();
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
      key: devices,
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
          ListFooter(busy: loadingMore, failure: pageFailure, onRetry: _more),
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
            Text(
              problem!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
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
