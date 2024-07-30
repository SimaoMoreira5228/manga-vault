import {api} from '$lib/axios.server';
import type {PageServerLoad} from './$types';
import {getUser} from '$lib/utils.server';

type Scrapers = {
    id: string;
    name: string;
    img_url: string;
};

export const load: PageServerLoad = async ({cookies}) => {
    const token = cookies.get('token');
    const user = await getUser(cookies);

    let scrapers: Scrapers[] = [];
    if (user) {
        scrapers = await api
            .get(`api/scrapers`, {
                headers: {
                    Authorization: token
                }
            })
            .then((res) => res.data);
    }

    return {scrapers};
};
