class apiClass {
	private readonly apiUrl: string = import.meta.env.VITE_API_URL;

	async get<T>(url: string, options?: RequestInit): Promise<T> {
		const res = await fetch(`${this.apiUrl}${url}`, options);

		if (!res.ok) {
			throw new Error(res.statusText);
		}

		return res.json();
	}

	async post<T>(url: string, body: unknown, options?: RequestInit): Promise<T> {
		const res = await fetch(`${this.apiUrl}${url}`, {
			method: 'POST',
			body: JSON.stringify(body),
			headers: options?.headers || {
				'Content-Type': 'application/json'
			},
			...options
		});

		if (!res.ok) {
			throw new Error(await res.text() || res.statusText);
		}

		return res.json();
	}

	async put<T>(url: string, body: unknown, options?: RequestInit): Promise<T> {
		const res = await fetch(`${this.apiUrl}${url}`, {
			method: 'PUT',
			body: JSON.stringify(body),
			headers: options?.headers || {
				'Content-Type': 'application/json'
			},
			...options
		});

		if (!res.ok) {
			throw new Error(res.statusText);
		}

		return res.json();
	}

	async delete<T>(url: string, options?: RequestInit): Promise<T> {
		const res = await fetch(`${this.apiUrl}${url}`, {
			method: 'DELETE',
			...options
		});

		if (!res.ok) {
			throw new Error(res.statusText);
		}

		return res.json();
	}
}

export const api = new apiClass();
export const fileUrl = (id: string) => `${import.meta.env.VITE_API_URL}files/image/${id}`;
