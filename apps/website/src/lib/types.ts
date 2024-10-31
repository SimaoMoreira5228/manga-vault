export enum SortType {
    TITLE = 'title',
    CHAPTERS = 'unread_chapters'
}

export type FavoritesMangaItem = {
    id: number;
    title: string;
    url: string;
    img_url: string;
    scraper: string;
    chapters_number: number;
    read_chapters_number: number;
    created_at: string;
    updated_at: string;
};

export type MangaItem = {
    id: number;
    title: string;
    url: string;
    img_url: string;
    scraper: string;
    created_at: string;
    updated_at: string;
};

export type allSearchedMangaItems = {
    scraper: string;
    mangas: MangaItem[];
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
    scraped?: boolean;
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

export type User = {
    id: number;
    username: string;
    image_id: string;
};

export type WsMessage = {
    msg_type: string;
    content: Content;
};

export type Content = {
    user_id: number;
    category_id: number | undefined;
};

export type WsResponse = {
    msg_type: string;
    content: FavoritesMangaItem | undefined;
    error: string | undefined;
};
