import 'dart:async';
import 'dart:convert';
import 'dart:io';

class VaultEvents {
  VaultEvents._();

  static final VaultEvents instance = VaultEvents._();

  final _listeners = <void Function(String workId)>{};
  HttpClient? _client;
  StreamSubscription<String>? _subscription;
  Timer? _retry;
  String? _baseUrl;
  String? _token;
  var _generation = 0;

  bool get isActive => _subscription != null || _retry != null;

  void start({required String baseUrl, required String token}) {
    if (isActive && _baseUrl == baseUrl && _token == token) return;
    stop();
    _baseUrl = baseUrl;
    _token = token;
    _loop();
  }

  void stop() {
    _generation += 1;
    _retry?.cancel();
    _retry = null;
    _subscription?.cancel();
    _subscription = null;
    _client?.close();
    _client = null;
  }

  void subscribe(void Function(String workId) listener) =>
      _listeners.add(listener);

  void unsubscribe(void Function(String workId) listener) =>
      _listeners.remove(listener);

  Future<void> _loop() async {
    final generation = ++_generation;
    final client = HttpClient();
    _client = client;
    try {
      while (isActive && generation == _generation) {
        try {
          final request = await client
              .openUrl('GET', Uri.parse('$_baseUrl/api/events'))
              .timeout(const Duration(seconds: 15));
          request.headers.set(
            HttpHeaders.authorizationHeader,
            'Bearer $_token',
          );
          request.headers.set(HttpHeaders.acceptHeader, 'text/event-stream');
          final response = await request.close();
          if (response.statusCode != 200) {
            throw HttpException(
              'event stream refused (${response.statusCode})',
            );
          }
          _subscription = response
              .transform(utf8.decoder)
              .transform(const LineSplitter())
              .listen((line) {
                if (generation != _generation) return;
                _handleLine(line);
              });
          await _subscription!.asFuture<void>();
        } catch (_) {
          // dropped or refused: fall through to the retry delay
        }
        if (!isActive || generation != _generation) break;
        await Future<void>.delayed(const Duration(seconds: 5));
      }
    } finally {
      client.close();
      if (_client == client) _client = null;
    }
  }

  void _handleLine(String line) {
    if (!line.startsWith('data:')) return;
    try {
      final event =
          jsonDecode(line.substring(5).trim()) as Map<String, dynamic>;
      if (event['type'] != 'work_refreshed') return;
      final workId = event['work_id'] as String?;
      if (workId == null) return;
      for (final listener in Set.of(_listeners)) {
        listener(workId);
      }
    } catch (_) {
      // malformed payload: ignore
    }
  }
}
