import 'dart:convert';
import 'dart:io';

import 'package:path_provider/path_provider.dart';

class SessionStore {
  SessionStore._();

  static Future<File> _file() async {
    final support = await getApplicationSupportDirectory();
    return File('${support.path}/remote.json');
  }

  static Future<Map<String, dynamic>> read() async {
    final file = await _file();
    if (!file.existsSync()) return <String, dynamic>{};
    try {
      return (jsonDecode(file.readAsStringSync()) as Map)
          .cast<String, dynamic>();
    } catch (_) {
      return <String, dynamic>{};
    }
  }

  static Future<void> saveLocal() => _write({'kind': 'local'});

  static Future<void> saveLink({
    required String baseUrl,
    required String token,
  }) => _write({'kind': 'link', 'base_url': baseUrl, 'token': token});

  static Future<void> saveRemote({
    required String baseUrl,
    required String token,
  }) => _write({'kind': 'remote', 'base_url': baseUrl, 'token': token});

  static Future<void> _write(Map<String, dynamic> data) async {
    final file = await _file();
    final temp = File('${file.path}.tmp');
    await temp.writeAsString(jsonEncode(data), flush: true);
    if (await file.exists()) await file.delete();
    await temp.rename(file.path);
  }
}