import 'package:flutter/material.dart';

import '../service/remote_error.dart';
import '../service/remote_service.dart';

Future<RemoteService?> showServerSignIn(
  BuildContext context, {
  String title = 'Use a server account',
  String submitLabel = 'Sign in',
  String failureLabel = 'Login failed',
}) => showDialog<RemoteService>(
  context: context,
  builder: (context) => _ServerSignIn(
    title: title,
    submitLabel: submitLabel,
    failureLabel: failureLabel,
  ),
);

enum _FailureField { username, invite, form }

class _Failure {
  const _Failure(this.field, this.message);

  final _FailureField field;
  final String message;
}

_Failure _describeFailure(Object failure, {required bool registering}) {
  if (failure is! RemoteApiException) {
    return _Failure(_FailureField.form, '$failure');
  }
  if (registering && failure.code == 'username_taken') {
    return const _Failure(_FailureField.username, 'That username is taken.');
  }
  if (registering && failure.code == 'invite_required') {
    return const _Failure(
      _FailureField.invite,
      'An invite code is required to register.',
    );
  }
  if (registering && failure.code == 'invalid_invite') {
    return const _Failure(
      _FailureField.invite,
      'That invite code is invalid or already used.',
    );
  }
  switch (failure.code) {
    case 'bad_credentials':
      return const _Failure(_FailureField.form, 'Wrong username or password.');
    case 'registration_closed':
      return const _Failure(
        _FailureField.form,
        'This server is not accepting new accounts.',
      );
    case 'too_many_attempts':
    case 'too_many_requests':
      final seconds = failure.retryAfterSeconds;
      return _Failure(
        _FailureField.form,
        seconds == null
            ? failure.message
            : 'Too many attempts. Retry in ${seconds}s.',
      );
    default:
      return _Failure(_FailureField.form, failure.message);
  }
}

class _ServerSignIn extends StatefulWidget {
  const _ServerSignIn({
    required this.title,
    required this.submitLabel,
    required this.failureLabel,
  });

  final String title;
  final String submitLabel;
  final String failureLabel;

  @override
  State<_ServerSignIn> createState() => _ServerSignInState();
}

class _ServerSignInState extends State<_ServerSignIn> {
  final baseUrl = TextEditingController();
  final username = TextEditingController();
  final password = TextEditingController();
  final inviteCode = TextEditingController();
  String? registration;
  String? usernameError;
  String? inviteError;
  var registering = false;
  var busy = false;

  bool get canRegister => registration != 'closed';

  Future<void> _probeRegistration() async {
    final url = baseUrl.text.trim();
    if (url.isEmpty) return;
    var mode = 'open';
    try {
      mode = await RemoteService.registrationMode(url);
    } catch (_) {}
    if (!mounted) return;
    setState(() => registration = mode);
  }

  void _reportFailure(Object failure) {
    final described = _describeFailure(failure, registering: registering);
    switch (described.field) {
      case _FailureField.username:
        setState(() => usernameError = described.message);
      case _FailureField.invite:
        setState(() {
          if (registration != 'invite') registration = 'invite';
          inviteError = described.message;
        });
      case _FailureField.form:
        setState(() {
          if (failure is RemoteApiException &&
              failure.code == 'registration_closed') {
            registration = 'closed';
          }
        });
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('${widget.failureLabel}: ${described.message}'),
          ),
        );
    }
  }

  Future<void> _submit() async {
    setState(() {
      busy = true;
      usernameError = null;
      inviteError = null;
    });
    try {
      final url = baseUrl.text.trim();
      final service = registering
          ? await RemoteService.register(
              baseUrl: url,
              username: username.text,
              password: password.text,
              inviteCode: inviteCode.text,
            )
          : await RemoteService.login(
              baseUrl: url,
              username: username.text,
              password: password.text,
            );
      if (!mounted) return;
      Navigator.of(context).pop(service);
    } catch (e) {
      if (!mounted) return;
      setState(() => busy = false);
      _reportFailure(e);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.title),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: baseUrl,
            onSubmitted: (_) => _probeRegistration(),
            decoration: const InputDecoration(
              hintText: 'https://server.example.org',
            ),
          ),
          TextField(
            controller: username,
            decoration: InputDecoration(
              hintText: 'Username',
              errorText: usernameError,
            ),
          ),
          TextField(
            controller: password,
            obscureText: true,
            decoration: const InputDecoration(hintText: 'Password'),
          ),
          if (registering && registration == 'invite')
            TextField(
              controller: inviteCode,
              decoration: InputDecoration(
                hintText: 'Invite code',
                errorText: inviteError,
              ),
            ),
        ],
      ),
      actions: [
        if (canRegister)
          TextButton(
            onPressed: busy
                ? null
                : () => setState(() => registering = !registering),
            child: Text(
              registering
                  ? 'Already registered? Sign in'
                  : registration == 'invite'
                  ? 'Have an invite code? Create an account'
                  : 'No account yet? Create one',
            ),
          ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: busy || (registering && !canRegister) ? null : _submit,
          child: Text(registering ? 'Create account' : widget.submitLabel),
        ),
      ],
    );
  }
}
