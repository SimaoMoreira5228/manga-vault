import { client } from "$lib/graphql/client";
import { gql } from "@urql/svelte";

export async function load({ params }: { params: { chapter_id: string } }) {
    const response = await client
			.query(
				gql`
					query getChapterInfo($chapterId: Int!) {
						chapters {
							chapter(id: $chapterId) {
								images
								nextChapter {
									id
								}
								previousChapter {
									id
								}
								scraper {
									refererUrl
								}
							}
						}
					}
				`,
				{ chapterId: params.chapter_id ? parseInt(params.chapter_id) : null }
			)
			.toPromise();

		if (response.error) {
			console.error('Failed to load chapter', response.error);
			return { status: 500, error: new Error('Failed to load chapter') };
		}

        return {
            nextChapter: response.data.chapters.chapter?.nextChapter?.id || null,
			previousChapter: response.data.chapters.chapter?.previousChapter?.id || null,
			imageUrls: response.data.chapters.chapter?.images || [],
			refererUrl: response.data.chapters.chapter?.scraper?.refererUrl || null,
        }
}