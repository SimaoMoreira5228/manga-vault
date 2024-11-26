use connection::Connection;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use scraper_core::PLUGIN_MANAGER;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::entities::prelude::{Chapters, FavoriteMangas, Mangas, ReadChapters};
use crate::websocket::{Content, MangaResponse, SyncFavoriteMangasResponse};

pub async fn sync_favorite_mangas_from_category(
	write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
	content: Content,
	db: Connection,
) {
	if content.category_id.is_none() {
		let response = SyncFavoriteMangasResponse {
			msg_type: "sync-category".to_string(),
			content: None,
			error: Some("Category id is required".to_string()),
		};
		write
			.send(Message::Binary(serde_json::to_vec(&response).unwrap()))
			.await
			.unwrap();
		return;
	}

	let user_favorite_mangas = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.eq(content.user_id))
		.filter(crate::entities::favorite_mangas::Column::CategoryId.eq(content.category_id.unwrap()))
		.all(&db)
		.await
		.unwrap();

	let mut favorite_mangas: Vec<crate::entities::mangas::Model> = Vec::new();

	for favorite_manga in user_favorite_mangas {
		let manga: crate::entities::mangas::Model = Mangas::find()
			.filter(crate::entities::mangas::Column::Id.eq(favorite_manga.manga_id))
			.one(&db)
			.await
			.unwrap()
			.unwrap();

		favorite_mangas.push(manga);
	}

	for favorite_manga in favorite_mangas {
		let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&favorite_manga.scraper);

		let plugin = if let Some(p) = plugin {
			p
		} else {
			return write
				.send(Message::Binary(
					serde_json::to_vec(&SyncFavoriteMangasResponse {
						msg_type: "sync-all".to_string(),
						content: None,
						error: Some("Invalid scraper".to_string()),
					})
					.unwrap(),
				))
				.await
				.unwrap();
		};

		let manga_page = plugin.scrape_manga(&favorite_manga.url);

		if manga_page.is_err() {
			let response = SyncFavoriteMangasResponse {
				msg_type: "sync-category".to_string(),
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
		let read_chapters = ReadChapters::find()
			.filter(crate::entities::read_chapters::Column::UserId.eq(content.user_id))
			.filter(crate::entities::read_chapters::Column::MangaId.eq(favorite_manga.id))
			.count(&db)
			.await
			.unwrap();

		for chapter in chapters {
			let db_chapter: Option<crate::entities::chapters::Model> = Chapters::find()
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

				let inserted_chaper = active_model_chapter.insert(&db).await;

				if inserted_chaper.is_err() {
					let response = SyncFavoriteMangasResponse {
						msg_type: "sync-category".to_string(),
						content: None,
						error: Some("Error inserting chapter".to_string()),
					};
					write
						.send(Message::Binary(serde_json::to_vec(&response).unwrap()))
						.await
						.unwrap();
					continue;
				}
			}
		}

		let mut favorite_manga_active = favorite_manga.clone().into_active_model();
		favorite_manga_active.img_url = Set(manga_page.img_url.clone());
		favorite_manga_active.updated_at = Set(chrono::Utc::now().naive_utc().to_string());
		let new_favorite = favorite_manga_active.update(&db).await.unwrap();

		let response = SyncFavoriteMangasResponse {
			msg_type: "sync-category".to_string(),
			content: Some(MangaResponse {
				title: manga_page.title,
				url: new_favorite.url,
				img_url: manga_page.img_url,
				chapters_number: chapters_count as u64,
				read_chapters_number: read_chapters,
				created_at: new_favorite.created_at,
				id: new_favorite.id,
				scraper: new_favorite.scraper.to_string(),
				updated_at: new_favorite.updated_at,
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
