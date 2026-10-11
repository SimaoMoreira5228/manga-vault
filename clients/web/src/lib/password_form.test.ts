import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';
import {
	checkOperatorReset,
	checkPasswordForm,
	MIN_PASSWORD_LENGTH,
	passwordChangeNotice,
} from './password_form.ts';

const registration = readFileSync(
	fileURLToPath(new URL('../../../../crates/application/src/registration.rs', import.meta.url)),
	'utf8',
);

function serverMinimumLength(): number {
	const declaration = registration.match(/pub const MIN_PASSWORD_LENGTH: usize = (\d+);/);
	assert.notEqual(declaration, null, 'MIN_PASSWORD_LENGTH is gone from the application crate');
	return Number(declaration?.[1]);
}

describe('change password form', () => {
	it('applies the same minimum length the server enforces', () => {
		assert.equal(MIN_PASSWORD_LENGTH, serverMinimumLength());
	});

	it('asks for the current password before anything else', () => {
		assert.deepEqual(checkPasswordForm({ current: '', next: 'x', confirm: 'x' }), {
			field: 'current',
			message: 'Enter your current password.',
		});
	});

	it('flags the new password when it is too short', () => {
		const failure = checkPasswordForm({
			current: 'old password here',
			next: 'a'.repeat(MIN_PASSWORD_LENGTH - 1),
			confirm: 'a'.repeat(MIN_PASSWORD_LENGTH - 1),
		});
		assert.equal(failure?.field, 'next');
		assert.match(failure?.message ?? '', new RegExp(`${MIN_PASSWORD_LENGTH} characters`));
	});

	it('refuses a new password that repeats the current one', () => {
		const shared = 'the very same passphrase';
		assert.equal(
			checkPasswordForm({ current: shared, next: shared, confirm: shared })?.field,
			'next',
		);
	});

	it('flags the repeat field when the two new passwords differ', () => {
		assert.equal(
			checkPasswordForm({
				current: 'old password here',
				next: 'a brand new passphrase',
				confirm: 'a different passphrase',
			})?.field,
			'confirm',
		);
	});

	it('accepts a well formed change', () => {
		assert.equal(
			checkPasswordForm({
				current: 'old password here',
				next: 'a brand new passphrase',
				confirm: 'a brand new passphrase',
			}),
			null,
		);
	});
});

describe('operator reset form', () => {
	it('will not submit without an account to reset', () => {
		assert.equal(
			checkOperatorReset('   ', 'a brand new passphrase', 'a brand new passphrase')?.field,
			'form',
		);
	});

	it('applies the same minimum length and repeat check', () => {
		assert.equal(checkOperatorReset('reader', 'short', 'short')?.field, 'next');
		assert.equal(
			checkOperatorReset('reader', 'a brand new passphrase', 'something else')?.field,
			'confirm',
		);
		assert.equal(
			checkOperatorReset('reader', 'a brand new passphrase', 'a brand new passphrase'),
			null,
		);
	});
});

describe('what the client tells the user afterwards', () => {
	it('says plainly how many other devices were signed out', () => {
		assert.equal(passwordChangeNotice(0), 'Password changed. No other device was signed in.');
		assert.equal(passwordChangeNotice(1), 'Password changed. Signed out of 1 other device.');
		assert.equal(passwordChangeNotice(3), 'Password changed. Signed out of 3 other devices.');
	});
});
