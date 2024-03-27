export type MangaItem = {
	id: number;
	title: string;
	url: string;
	img_url: string;
	scrapper: string;
	chapters_number: number;
	read_chapters_number: number;
	created_at: string;
	updated_at: string;
};

export type MangaSource = {
	id: string;
	name: string;
	img_url: string;
};

export type Chapter = {
	id: number;
	title: string;
	url: string;
	created_at: string;
	updated_at: string;
	manga_id: number;
};

export type MangaPage = {
	title: string;
	url: string;
	img_url: string;
	alternative_names: string[];
	authors: string[];
	artists: string[] | null;
	status: string;
	type: string | null;
	release_date: string | null;
	description: string;
	genres: string[];
	chapters: Chapter[];
};

export type Category = {
	id: number;
	name: string;
	user_id: number;
	created_at: string;
};

export type ReadChapter = {
	id: number;
	user_id: number;
	chapter_id: number;
	created_at: string;
};