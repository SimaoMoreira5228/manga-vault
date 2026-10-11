import 'dart:convert';

class RemoteApiException implements Exception {
  RemoteApiException(
    this.statusCode,
    this.message, {
    this.code,
    this.retryAfterSeconds,
  });

  final int statusCode;
  final String message;
  final String? code;
  final int? retryAfterSeconds;

  @override
  String toString() => message;
}

RemoteApiException decodeRemoteApiException({
  required int statusCode,
  required String body,
  required String fallbackMessage,
  String? retryAfterHeader,
}) {
  Object? decoded;
  try {
    decoded = jsonDecode(body);
  } on FormatException {
    decoded = null;
  }
  final payload = decoded is Map<String, dynamic> ? decoded : const {};
  final error = payload['error'];
  final code = payload['code'];
  return RemoteApiException(
    statusCode,
    error is String ? error : fallbackMessage,
    code: code is String ? code : null,
    retryAfterSeconds: int.tryParse(retryAfterHeader ?? ''),
  );
}
