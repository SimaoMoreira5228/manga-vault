import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { cubicOut } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { themeStore } from './stores';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

type FlyAndScaleParams = {
	y?: number;
	x?: number;
	start?: number;
	duration?: number;
};

export const flyAndScale = (
	node: Element,
	params: FlyAndScaleParams = { y: -8, x: 0, start: 0.95, duration: 150 }
): TransitionConfig => {
	const style = getComputedStyle(node);
	const transform = style.transform === 'none' ? '' : style.transform;

	const scaleConversion = (valueA: number, scaleA: [number, number], scaleB: [number, number]) => {
		const [minA, maxA] = scaleA;
		const [minB, maxB] = scaleB;

		const percentage = (valueA - minA) / (maxA - minA);
		const valueB = percentage * (maxB - minB) + minB;

		return valueB;
	};

	const styleToString = (style: Record<string, number | string | undefined>): string => {
		return Object.keys(style).reduce((str, key) => {
			if (style[key] === undefined) return str;
			return str + `${key}:${style[key]};`;
		}, '');
	};

	return {
		duration: params.duration ?? 200,
		delay: 0,
		css: (t) => {
			const y = scaleConversion(t, [0, 1], [params.y ?? 5, 0]);
			const x = scaleConversion(t, [0, 1], [params.x ?? 0, 0]);
			const scale = scaleConversion(t, [0, 1], [params.start ?? 0.95, 1]);

			return styleToString({
				transform: `${transform} translate3d(${x}px, ${y}px, 0) scale(${scale})`,
				opacity: t
			});
		},
		easing: cubicOut
	};
};

export const toggleTheme = () => {
	themeStore.update((theme) => {
		if (theme === 'dark') {
			document.documentElement.classList.remove('dark');
			return 'light';
		} else {
			document.documentElement.classList.add('dark');
			return 'dark';
		}
	});
};

export const getTitle = (path: string) => {
	switch (true) {
		case path.includes('library/search'):
			return 'Search';
		case path.includes('library/sources'):
			return 'Sources';
		case path.includes('library'):
			return 'Library';
		case path.includes('/users/login'):
			return 'Login';
		case path.includes('/users/new'):
			return 'Register';
		default:
			return 'Home';
	}
};

export function normalizeTitles(title: string) {
	const words = title.toLowerCase().split(' ');
	const newTittle = words.map((word) => {
		return word.charAt(0).toUpperCase() + word.slice(1);
	});

	return newTittle.join(' ');
}

export function createButton(svg: string) {
	const button = document.createElement('button');
	button.innerHTML = svg;

	button.classList.add(
		'inline-flex',
		'items-center',
		'justify-center',
		'rounded-md',
		'text-sm',
		'font-medium',
		'whitespace-nowrap',
		'ring-offset-background',
		'transition-colors',
		'focus-visible:outline-none',
		'focus-visible:ring-2',
		'focus-visible:ring-ring',
		'focus-visible:ring-offset-2',
		'disabled:pointer-events-none',
		'disabled:opacity-50',
		'bg-primary',
		'text-primary-foreground',
		'hover:bg-primary/90',
		'h-10',
		'w-10',
		'min-w-[2.5rem]'
	);

	return button;
}
