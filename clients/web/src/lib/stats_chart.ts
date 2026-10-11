export interface DailyBar {
	day: string;
	count: number;
	height: number;
	label: string | null;
}

export function dayLabel(day: string): string {
	return day.length > 5 ? day.slice(5) : day;
}

export function dailyBars(daily: [string, number][], plotHeight: number): DailyBar[] {
	const days = [...daily].sort((a, b) => a[0].localeCompare(b[0]));
	const peak = days.reduce((best, [, count]) => (count > best ? count : best), 1);
	return days.map(([day, count], index) => ({
		day,
		count,
		height: (plotHeight * count) / peak,
		label: index % 5 === 0 || index === days.length - 1 ? dayLabel(day) : null,
	}));
}
