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

  List<String> get localLibraryPaths =>
      (_data['localLibraryPaths'] as List?)?.cast<String>() ?? [];

  Future<void> addLocalLibraryPath(String path) async {
    final paths = localLibraryPaths;
    if (!paths.contains(path)) {
      paths.add(path);
      _data['localLibraryPaths'] = paths;
      await _save();
    }
  }

  Future<void> removeLocalLibraryPath(String path) async {
    final paths = localLibraryPaths..remove(path);
    _data['localLibraryPaths'] = paths;
    await _save();
  }

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

  // Reader settings
  double get globalFontSize => (_data['fontSize'] as num?)?.toDouble() ?? 16.0;
  Future<void> setGlobalFontSize(double value) async {
    _data['fontSize'] = value;
    await _save();
  }

  double get globalLineHeight =>
      (_data['lineHeight'] as num?)?.toDouble() ?? 1.6;
  Future<void> setGlobalLineHeight(double value) async {
    _data['lineHeight'] = value;
    await _save();
  }

  double get globalImageMargin =>
      (_data['imageMargin'] as num?)?.toDouble() ?? 0.0;
  Future<void> setGlobalImageMargin(double value) async {
    _data['imageMargin'] = value;
    await _save();
  }

  double get globalImageGap => (_data['imageGap'] as num?)?.toDouble() ?? 0.0;
  Future<void> setGlobalImageGap(double value) async {
    _data['imageGap'] = value;
    await _save();
  }

  bool get pagedMode => _data['pagedMode'] == true;
  Future<void> setPagedMode(bool value) async {
    _data['pagedMode'] = value;
    await _save();
  }

  bool get rtlMode => _data['rtlMode'] == true;
  Future<void> setRtlMode(bool value) async {
    _data['rtlMode'] = value;
    await _save();
  }

  // Per-work overrides
  Map<String, dynamic>? workReaderSettings(String workId) {
    final overrides = (_data['workOverrides'] as Map?)?.cast<String, dynamic>();
    return overrides?[workId] as Map<String, dynamic>?;
  }

  Future<void> setWorkReaderSetting(
    String workId,
    String key,
    dynamic value,
  ) async {
    final overrides = (_data['workOverrides'] ??= {}) as Map;
    final workSettings = (overrides[workId] ??= {}) as Map;
    workSettings[key] = value;
    await _save();
  }

  double effectiveFontSize(String workId) =>
      workReaderSettings(workId)?['fontSize']?.toDouble() ?? globalFontSize;
  double effectiveLineHeight(String workId) =>
      workReaderSettings(workId)?['lineHeight']?.toDouble() ?? globalLineHeight;
  bool effectivePagedMode(String workId) =>
      workReaderSettings(workId)?['pagedMode'] as bool? ?? pagedMode;
  double effectiveImageMargin(String workId) =>
      workReaderSettings(workId)?['imageMargin']?.toDouble() ??
      globalImageMargin;
  double effectiveImageGap(String workId) =>
      workReaderSettings(workId)?['imageGap']?.toDouble() ?? globalImageGap;
}
