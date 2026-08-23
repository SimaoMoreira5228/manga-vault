import 'vault_service.dart';

class SyncReport {
	const SyncReport({required this.pushed, required this.pulled});

	final Map<String, dynamic> pushed;
	final Map<String, dynamic> pulled;
}

class SyncEngine {
	SyncEngine({required this.local, required this.remote});

	final VaultService local;
	final VaultService remote;

	Future<SyncReport> synchronize() async {
		final localState = await local.exportSyncState();
		await remote.applySyncState(localState);

		final remoteState = await remote.exportSyncState();
		await local.applySyncState(remoteState);

		return SyncReport(pushed: localState, pulled: remoteState);
	}
}
