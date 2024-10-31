use connection::Connection;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use scrapers::PLUGIN_MANAGER;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::entities::prelude::{Chapters, FavoriteMangas, Mangas};
use crate::websocket::{Content, SyncFavoriteMangasResponse};

pub async fn sync_all_favorite_mangas(
	write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
	content: Content,
	db: Connection,
) {
	let user_favorite_mangas = FavoriteMangas::find()
		.filter(crate::entities::favorite_mangas::Column::UserId.eq(content.user_id))
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
		let plugin = PLUGIN_MANAGER.get().unwrap().get_plugin(&favorite_manga.scraper).await;

		let plugin = if plugin.is_none() {
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
		} else {
			plugin.unwrap()
		};

		let manga_page = plugin.scrape_manga(&favorite_manga.url);

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

		for chapter in chapters {
			let db_chapter = Chapters::find()
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

				let res = active_model_chapter.insert(&db).await;

				if res.is_err() {
					let response = SyncFavoriteMangasResponse {
						msg_type: "sync-all".to_string(),
						content: None,
						error: Some("Error saving chapter".to_string()),
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
		let _ = favorite_manga_active.update(&db).await.unwrap();
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
