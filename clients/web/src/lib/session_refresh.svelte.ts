export const sessionEpoch = $state({ value: 0 });

export function refreshSessions(): void {
	sessionEpoch.value += 1;
}
