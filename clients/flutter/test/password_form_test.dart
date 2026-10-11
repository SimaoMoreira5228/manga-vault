import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:manga_vault/service/password_form.dart';

int _serverMinimumLength() {
  final source = File(
    '../../crates/application/src/registration.rs',
  ).readAsStringSync();
  final declaration = RegExp(
    r'pub const MIN_PASSWORD_LENGTH: usize = (\d+);',
  ).firstMatch(source);
  expect(declaration, isNotNull, reason: 'MIN_PASSWORD_LENGTH left the application crate');
  return int.parse(declaration!.group(1)!);
}

void main() {
  test('applies the same minimum length the server enforces', () {
    expect(minPasswordLength, _serverMinimumLength());
  });

  test('asks for the current password before anything else', () {
    expect(
      checkPasswordChange(current: '', next: 'x', repeat: 'x')?.field,
      'current',
    );
  });

  test('flags the new password when it is too short', () {
    final failure = checkPasswordChange(
      current: 'old password here',
      next: 'a' * (minPasswordLength - 1),
      repeat: 'a' * (minPasswordLength - 1),
    );
    expect(failure?.field, 'next');
    expect(failure?.message, contains('$minPasswordLength characters'));
  });

  test('refuses a new password that repeats the current one', () {
    const shared = 'the very same passphrase';
    expect(
      checkPasswordChange(current: shared, next: shared, repeat: shared)?.field,
      'next',
    );
  });

  test('flags the repeat field when the two new passwords differ', () {
    expect(
      checkPasswordChange(
        current: 'old password here',
        next: 'a brand new passphrase',
        repeat: 'a different passphrase',
      )?.field,
      'confirm',
    );
  });

  test('accepts a well formed change', () {
    expect(
      checkPasswordChange(
        current: 'old password here',
        next: 'a brand new passphrase',
        repeat: 'a brand new passphrase',
      ),
      isNull,
    );
  });

  test('says plainly how many other devices were signed out', () {
    expect(passwordChangeNotice(0), 'Password changed. No other device was signed in.');
    expect(passwordChangeNotice(1), 'Password changed. Signed out of 1 other device.');
    expect(passwordChangeNotice(3), 'Password changed. Signed out of 3 other devices.');
  });
}
