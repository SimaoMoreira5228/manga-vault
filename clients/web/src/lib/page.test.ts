import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { hasNext, pageCount, pageNumber, pageQuery } from './page.ts';

describe('page math', () => {
	it('keeps the query string offset-specific so every page is its own url', () => {
		assert.equal(pageQuery(0), 'offset=0&limit=50');
		assert.equal(pageQuery(50), 'offset=50&limit=50');
		assert.notEqual(pageQuery(0), pageQuery(50));
	});

	it('never asks for a negative offset', () => {
		assert.equal(pageQuery(-20), 'offset=0&limit=50');
	});

	it('counts pages from the offset the server echoed back', () => {
		assert.equal(pageNumber(0, 50), 1);
		assert.equal(pageNumber(50, 50), 2);
		assert.equal(pageNumber(199, 200), 1);
		assert.equal(pageNumber(200, 200), 2);
	});

	it('rounds the last short page up', () => {
		assert.equal(pageCount(0, 50), 1);
		assert.equal(pageCount(50, 50), 1);
		assert.equal(pageCount(51, 50), 2);
		assert.equal(pageCount(250, 50), 5);
	});

	it('stops paging once offset plus limit reaches the total', () => {
		assert.equal(hasNext(0, 50, 50), false);
		assert.equal(hasNext(0, 50, 51), true);
		assert.equal(hasNext(50, 50, 51), false);
	});
});
