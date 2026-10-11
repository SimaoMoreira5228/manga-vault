const int minPasswordLength = 8;

class PasswordFormFailure {
  const PasswordFormFailure(this.field, this.message);

  final String field;
  final String message;
}

PasswordFormFailure? checkPasswordChange({
  required String current,
  required String next,
  required String repeat,
}) {
  if (current.isEmpty) {
    return const PasswordFormFailure('current', 'Enter your current password.');
  }
  if (next.length < minPasswordLength) {
    return PasswordFormFailure('next', 'Use at least $minPasswordLength characters.');
  }
  if (next == current) {
    return const PasswordFormFailure(
      'next',
      'The new password must differ from the current one.',
    );
  }
  if (next != repeat) {
    return const PasswordFormFailure(
      'confirm',
      'The two new passwords do not match.',
    );
  }
  return null;
}

String passwordChangeNotice(int revokedSessions) {
  if (revokedSessions == 0) {
    return 'Password changed. No other device was signed in.';
  }
  final plural = revokedSessions == 1 ? 'device' : 'devices';
  return 'Password changed. Signed out of $revokedSessions other $plural.';
}
