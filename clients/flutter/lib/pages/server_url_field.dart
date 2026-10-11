import 'package:flutter/material.dart';

Uri? parseServerUrl(String raw) {
  final trimmed = raw.trim();
  if (trimmed.isEmpty) return null;
  if (RegExp(r'\s').hasMatch(trimmed)) return null;
  final withScheme = trimmed.contains('://') ? trimmed : 'http://$trimmed';
  final parsed = Uri.tryParse(withScheme);
  if (parsed == null || parsed.host.isEmpty || parsed.host.contains('%')) {
    return null;
  }
  return Uri(
    scheme: parsed.scheme,
    host: parsed.host,
    port: parsed.hasPort ? parsed.port : null,
  );
}

String normalizeServerUrl(String raw) => parseServerUrl(raw)?.toString() ?? raw.trim();

String? serverUrlError(String raw) {
  if (raw.trim().isEmpty) return 'Enter the address of your server';
  if (parseServerUrl(raw) == null) return 'That does not look like a server address';
  return null;
}

class ServerUrlField extends StatelessWidget {
  const ServerUrlField({super.key, required this.controller, this.onChanged});

  final TextEditingController controller;
  final ValueChanged<String>? onChanged;

  @override
  Widget build(BuildContext context) => TextField(
    controller: controller,
    onChanged: onChanged,
    autocorrect: false,
    keyboardType: TextInputType.url,
    decoration: const InputDecoration(
      labelText: 'Server address',
      hintText: 'http://127.0.0.1:8080',
      helperText: 'The scheme is added for you if you leave it out',
    ),
  );
}
