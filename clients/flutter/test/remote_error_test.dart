import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/service/remote_error.dart';

void main() {
  test('reads the machine-readable code out of an error body', () {
    final failure = decodeRemoteApiException(
      statusCode: 409,
      body:
          '{"error":"username `dup` is already taken","code":"username_taken"}',
      fallbackMessage: 'unused',
    );
    expect(failure.code, 'username_taken');
    expect(failure.message, 'username `dup` is already taken');
  });

  test(
    'falls back to the raw body when the response is not an error object',
    () {
      final failure = decodeRemoteApiException(
        statusCode: 502,
        body: '<html>bad gateway</html>',
        fallbackMessage: 'GET /api/me failed (502): <html>bad gateway</html>',
      );
      expect(failure.code, isNull);
      expect(failure.retryAfterSeconds, isNull);
      expect(
        failure.message,
        'GET /api/me failed (502): <html>bad gateway</html>',
      );
    },
  );

  test('surfaces retry-after seconds when the server sends one', () {
    final failure = decodeRemoteApiException(
      statusCode: 429,
      body: '{"error":"slow down","code":"too_many_requests"}',
      fallbackMessage: 'unused',
      retryAfterHeader: '42',
    );
    expect(failure.code, 'too_many_requests');
    expect(failure.retryAfterSeconds, 42);
  });
}
