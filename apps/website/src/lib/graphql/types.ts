export type ScrapeItem = {
	title: string;
	url: string;
	imgUrl?: string | null;
	mangaId?: number | null;
	novelId?: number | null;
};

export type ScraperType = "MANGA" | "NOVEL";
export type WorkType = ScraperType;

export type Scraper = {
	id: string;
	name: string;
	imageUrl?: string | null;
	refererUrl?: string | null;
	type?: ScraperType;
};

export type User = {
	id: number;
	username: string;
	imageId?: number | null;
};

export type Chapter = {
	createdAt: string;
	id: number;
	scanlationGroup?: string | null;
	title: string;
	updatedAt: string;
	url: string;
};

export type ScraperInfo = {
	id: string;
	name: string;
	imageUrl: string;
	refererUrl?: string | null;
	type: ScraperType;
};

export type Work = {
	id: number;
	title: string;
	url: string;
	imgUrl: string;
	scraper: string;
	createdAt?: string | null;
	updatedAt: string;
	alternativeNames: string[];
	authors: string[];
	artists: string[];
	status?: string | null;
	mangaType?: string | null;
	novelType?: string | null;
	releaseDate?: string | null;
	description?: string | null;
	genres?: string[] | null;
	scrapeScheduled: boolean;
	chapters?: Chapter[] | null;
	scraperInfo?: ScraperInfo;
	type?: WorkType;
};

export type WorkWithFavorite = Work & {
	isFavorite: boolean;
	favoriteId?: number | null;
	userReadChaptersAmount?: number | null;
	chaptersAmount?: number | null;
	categoryId?: number | null;
	userReadChapters?: { id: number; chapterId: number }[];
};

export type GetWorkOptions = {
	includeChapters?: boolean;
	includeFavorite?: boolean;
	includeRead?: boolean;
};

export type GetMangaOptions = GetWorkOptions;

export type MangaCardLike = {
	title: string;
	imgUrl?: string | null;
	chaptersAmount?: number;
	userReadChaptersAmount?: number;
	id?: number | null;
	type?: WorkType;
};
