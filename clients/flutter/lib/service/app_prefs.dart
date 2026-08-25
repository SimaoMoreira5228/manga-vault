import 'dart:convert';
import 'dart:io';

import 'package:path_provider/path_provider.dart';

class AppPrefs {
  static AppPrefs? _instance;
  final File _file;
  final Map<String, dynamic> _data;

  AppPrefs._(this._file, this._data);

  static Future<AppPrefs> instance() async {
    final existing = _instance;
    if (existing != null) return existing;
    final dir = await getApplicationSupportDirectory();
    final file = File('${dir.path}/app_prefs.json');
    Map<String, dynamic> data = {};
    if (await file.exists()) {
      try {
        data = (jsonDecode(await file.readAsString()) as Map)
            .cast<String, dynamic>();
      } catch (_) {
        data = {};
      }
    }
    final prefs = AppPrefs._(file, data);
    _instance = prefs;
    return prefs;
  }

  bool get chapterNotifications => _data['chapterNotifications'] == true;

  Future<void> setChapterNotifications(bool value) async {
    _data['chapterNotifications'] = value;
    await _save();
  }

  double? positionFor(String chapterId) {
    final positions = (_data['positions'] as Map?)?.cast<String, dynamic>();
    final raw = positions?[chapterId];
    return raw is num ? raw.toDouble() : null;
  }

  Future<void> setPosition(String chapterId, double fraction) async {
    final positions = ((_data['positions'] ??= {}) as Map)
        .cast<String, dynamic>();
    if (fraction >= 0.98 || fraction <= 0.01) {
      positions.remove(chapterId);
    } else {
      positions[chapterId] = fraction;
    }
    await _save();
  }

  Future<void> _save() async {
    await _file.writeAsString(jsonEncode(_data));
  }
}
