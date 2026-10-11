import 'package:flutter/material.dart';

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

  Future<void> _submit() async {
    setState(() => busy = true);
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
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('${widget.failureLabel}: $e')));
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
            decoration: const InputDecoration(hintText: 'Username'),
          ),
          TextField(
            controller: password,
            obscureText: true,
            decoration: const InputDecoration(hintText: 'Password'),
          ),
          if (registering && registration == 'invite')
            TextField(
              controller: inviteCode,
              decoration: const InputDecoration(hintText: 'Invite code'),
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
