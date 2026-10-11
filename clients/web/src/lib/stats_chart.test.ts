import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { dailyBars, dayLabel } from './stats_chart.ts';

describe('read activity chart', () => {
	it('puts the days in calendar order even though the api sends newest first', () => {
		const bars = dailyBars(
			[
				['2026-08-03', 1],
				['2026-08-01', 2],
				['2026-08-02', 3],
			],
			120,
		);
		assert.deepEqual(
			bars.map((bar) => bar.day),
			['2026-08-01', '2026-08-02', '2026-08-03'],
		);
	});

	it('scales the busiest day to the full plot height', () => {
		const bars = dailyBars(
			[
				['2026-08-01', 5],
				['2026-08-02', 10],
				['2026-08-03', 0],
			],
			120,
		);
		assert.equal(bars[1].height, 120);
		assert.equal(bars[0].height, 60);
		assert.equal(bars[2].height, 0);
	});

	it('keeps a floor on the peak so an idle stretch yields no bars, not NaN', () => {
		const bars = dailyBars(
			[
				['2026-08-01', 0],
				['2026-08-02', 0],
			],
			120,
		);
		assert.deepEqual(
			bars.map((bar) => bar.height),
			[0, 0],
		);
	});

	it('labels every fifth day plus the most recent one', () => {
		const daily = Array.from(
			{ length: 7 },
			(_, index) => [`2026-08-0${index + 1}`, index] as [string, number],
		);
		assert.deepEqual(
			dailyBars(daily, 120).map((bar) => bar.label),
			['08-01', null, null, null, null, '08-06', '08-07'],
		);
	});

	it('falls back to the raw day when it carries no month prefix', () => {
		assert.equal(dayLabel('2026-08-09'), '08-09');
		assert.equal(dayLabel('0809'), '0809');
	});
});
