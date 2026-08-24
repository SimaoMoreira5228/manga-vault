type RefreshListener = (workId: string) => void;

const listeners = new Set<RefreshListener>();
let source: EventSource | null = null;

function ensureSource() {
	if (source || typeof window === 'undefined') return;
	source = new EventSource('/api/events');
	source.onmessage = (message) => {
		try {
			const event = JSON.parse(message.data) as { type?: string; work_id?: string };
			if (event.type !== 'work_refreshed' || !event.work_id) return;
			for (const listener of listeners) listener(event.work_id);
		} catch {
			// malformed payload: ignore
		}
	};
	source.onerror = () => {
		source?.close();
		source = null;
		setTimeout(ensureSource, 5000);
	};
}

export function onWorkRefreshed(listener: RefreshListener): () => void {
	if (typeof window === 'undefined') return () => {};
	ensureSource();
	listeners.add(listener);
	return () => listeners.delete(listener);
}
