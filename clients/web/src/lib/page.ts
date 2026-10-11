export const PAGE_SIZE = 50;
export const MAX_PAGE_SIZE = 200;

export function pageQuery(offset: number, limit = PAGE_SIZE): string {
	return `offset=${Math.max(0, Math.trunc(offset))}&limit=${limit}`;
}

export function pageNumber(offset: number, limit: number): number {
	return Math.floor(offset / limit) + 1;
}

export function pageCount(total: number, limit: number): number {
	return Math.max(1, Math.ceil(total / limit));
}

export function hasNext(offset: number, limit: number, total: number): boolean {
	return offset + limit < total;
}
