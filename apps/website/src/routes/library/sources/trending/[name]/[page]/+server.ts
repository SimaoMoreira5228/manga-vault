import {error, json} from '@sveltejs/kit';
import type {RequestHandler} from './$types';
import type {FavoritesMangaItem} from '$lib/types';
import {api} from '$lib/axios.server';

export const GET: RequestHandler = async ({cookies, params}) => {
    const token = cookies.get('token');

    const mangaItems: FavoritesMangaItem[] = await api
        .get(`api/scrapers/${params.name}/trending/${params.page}`, {
            headers: {Authorization: token}
        })
        .then((res) => res.data);

    if (mangaItems.length === 0) {
        return error(404, 'Not found');
    }

    return json(mangaItems);
};
