use crate::websocket::{Content, MangaResponse, SyncFavoriteMangasResponse};
use connection::Connection;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub async fn sync_all_favorite_mangas(
	write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
	content: Content,
	db: Connection,
) {
	let favorite_mangas_subquery = crate::entities::favorite_mangas::Entity::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.eq(content.user_id))
		.into_query();

	let favorite_mangas: Vec<crate::entities::mangas::Model> = crate::entities::mangas::Entity::find()
		.filter(crate::entities::mangas::Column::Id.in_subquery(favorite_mangas_subquery))
		.all(&db)
		.await
		.unwrap();

	for favorite_manga in favorite_mangas {
		let scrapper_type = scrappers::get_scrapper_type(&favorite_manga.scrapper);
		let scrapper = scrappers::Scrapper::new(&scrapper_type);
		let manga_page = scrapper.scrape_manga(&favorite_manga.url).await;

		if manga_page.is_err() {
			let response = SyncFavoriteMangasResponse {
				msg_type: "sync-all".to_string(),
				content: None,
				error: Some("Error scraping manga".to_string()),
			};
			write
				.send(Message::Binary(serde_json::to_vec(&response).unwrap()))
				.await
				.unwrap();
			continue;
		}

		let manga_page = manga_page.unwrap();
		let chapters = manga_page.chapters;
		let chapters_count = chapters.len();
		let mut read_chapters: Vec<i32> = vec![];

		for chapter in chapters {
			let db_chapter: Option<crate::entities::chapters::Model> = crate::entities::chapters::Entity::find()
				.filter(crate::entities::chapters::Column::MangaId.eq(favorite_manga.id))
				.filter(crate::entities::chapters::Column::Url.eq(&chapter.url))
				.one(&db)
				.await
				.unwrap();

			if db_chapter.is_none() {
				let active_model_chapter = crate::entities::chapters::ActiveModel {
					title: Set(chapter.title),
					url: Set(chapter.url),
					manga_id: Set(favorite_manga.id),
					created_at: Set(chrono::Utc::now().naive_utc().to_string()),
					updated_at: Set(chrono::Utc::now().naive_utc().to_string()),
					..Default::default()
				};

				//TODO: treat this
				let _ = active_model_chapter.insert(&db).await;

				let read_chapter: Option<crate::entities::read_chapters::Model> = crate::entities::read_chapters::Entity::find()
					.filter(crate::entities::read_chapters::Column::UserId.eq(content.user_id))
					.filter(crate::entities::read_chapters::Column::ChapterId.eq(db_chapter.unwrap().id))
					.one(&db)
					.await
					.unwrap();

				if read_chapter.is_some() {
					read_chapters.push(read_chapter.unwrap().chapter_id);
				}
			}
		}

		// TODO: maybe update the manga (img_url, title, etc)

		drop(scrapper);

		let response = SyncFavoriteMangasResponse {
			msg_type: "sync-all".to_string(),
			content: Some(MangaResponse {
				title: manga_page.title,
				url: favorite_manga.url,
				img_url: manga_page.img_url,
				chapters_count,
				read_chapters: read_chapters.len(),
			}),
			error: None,
		};

		write
			.send(Message::Binary(serde_json::to_vec(&response).unwrap()))
			.await
			.unwrap();
	}

	let response = SyncFavoriteMangasResponse {
		msg_type: "close-connection".to_string(),
		content: None,
		error: None,
	};

	write
		.send(Message::Binary(serde_json::to_vec(&response).unwrap()))
		.await
		.unwrap();
}
