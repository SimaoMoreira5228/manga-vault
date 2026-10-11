export const MIN_PASSWORD_LENGTH = 8;

export type PasswordField = 'current' | 'next' | 'confirm';

export interface PasswordFormInput {
	current: string;
	next: string;
	confirm: string;
}

export interface PasswordFormFailure {
	field: PasswordField | 'form';
	message: string;
}

export function checkPasswordForm(input: PasswordFormInput): PasswordFormFailure | null {
	if (input.current.length === 0) {
		return { field: 'current', message: 'Enter your current password.' };
	}
	if (input.next.length < MIN_PASSWORD_LENGTH) {
		return {
			field: 'next',
			message: `Use at least ${MIN_PASSWORD_LENGTH} characters.`,
		};
	}
	if (input.next === input.current) {
		return { field: 'next', message: 'The new password must differ from the current one.' };
	}
	if (input.next !== input.confirm) {
		return { field: 'confirm', message: 'The two new passwords do not match.' };
	}
	return null;
}

export function checkOperatorReset(
	username: string,
	next: string,
	confirm: string,
): PasswordFormFailure | null {
	if (username.trim().length === 0) {
		return { field: 'form', message: 'Name the account to reset.' };
	}
	if (next.length < MIN_PASSWORD_LENGTH) {
		return {
			field: 'next',
			message: `Use at least ${MIN_PASSWORD_LENGTH} characters.`,
		};
	}
	if (next !== confirm) {
		return { field: 'confirm', message: 'The two passwords do not match.' };
	}
	return null;
}

export function passwordChangeNotice(revokedSessions: number): string {
	if (revokedSessions === 0) {
		return 'Password changed. No other device was signed in.';
	}
	const plural = revokedSessions === 1 ? 'device' : 'devices';
	return `Password changed. Signed out of ${revokedSessions} other ${plural}.`;
}
