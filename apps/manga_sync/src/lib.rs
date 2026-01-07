use chrono::Utc;
use database_connection::Database;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum SyncError {
	#[error("Manga {manga_id} not found")]
	MangaNotFound {
		manga_id: i32,
	},

	#[error("Scraper {scraper_name} not found")]
	ScraperNotFound {
		scraper_name: String,
	},

	#[error("Database error: {0}")]
	Db(#[from] sea_orm::DbErr),

	#[error("URL parse error: {0}")]
	UrlParse(#[from] url::ParseError),

	#[error("Scraper plugin '{scraper_name}' has legacy URLs but no base URL")]
	ScraperMissingBaseUrl {
		scraper_name: String,
	},

	#[error("Invalid base URL: {base_url}")]
	InvalidBaseUrl {
		base_url: String,
	},

	#[error("Invalid manga URL: {url}")]
	InvalidMangaUrl {
		url: String,
	},

	#[error("Failed to set URL host to '{host}': {source}")]
	UrlSetHost {
		host: String,
		#[source]
		source: url::ParseError,
	},

	#[error("Invalid port '{port}': {source}")]
	InvalidPort {
		port: String,
		#[source]
		source: std::num::ParseIntError,
	},

	#[error("Scraper error: {message}")]
	ScraperError {
		message: String,
	},
}

pub async fn sync_manga_by_id(
	db: &Database,
	scraper_manager: &scraper_core::ScraperManager,
	manga_id: i32,
) -> Result<(), SyncError> {
	let manga = database_entities::mangas::Entity::find_by_id(manga_id)
		.one(&db.conn)
		.await?
		.ok_or_else(|| SyncError::MangaNotFound { manga_id })?;

	sync_manga_with_scraper(db, scraper_manager, manga_id, &manga.scraper).await
}

pub async fn sync_manga_with_scraper(
	db: &Database,
	scraper_manager: &scraper_core::ScraperManager,
	manga_id: i32,
	scraper_name: &str,
) -> Result<(), SyncError> {
	let mut manga = database_entities::mangas::Entity::find_by_id(manga_id)
		.one(&db.conn)
		.await?
		.ok_or_else(|| SyncError::MangaNotFound { manga_id })?;

	let plugin = scraper_manager
		.get_plugin(scraper_name)
		.await
		.ok_or_else(|| SyncError::ScraperNotFound {
			scraper_name: scraper_name.to_string(),
		})?;

	let plugin_info = plugin
		.get_info()
		.await
		.map_err(|e| SyncError::ScraperError { message: e.to_string() })?;

	if let Some(legacy_urls) = plugin_info.legacy_urls {
		let base_url = plugin_info.base_url.ok_or_else(|| SyncError::ScraperMissingBaseUrl {
			scraper_name: scraper_name.to_string(),
		})?;

		let canonical_host = host_from_base(&base_url)
			.ok_or_else(|| SyncError::InvalidBaseUrl {
				base_url: base_url.clone(),
			})?
			.to_lowercase();

		let legacy_hosts: Vec<String> = legacy_urls
			.iter()
			.filter_map(|d| Url::parse(d).ok().and_then(|u| u.host_str().map(|s| s.to_lowercase())))
			.collect();

		let parsed_manga_url = Url::parse(&manga.url)?;
		let manga_host = parsed_manga_url
			.host_str()
			.ok_or_else(|| SyncError::InvalidMangaUrl { url: manga.url.clone() })?
			.to_lowercase();

		if legacy_hosts.iter().any(|h| h == &manga_host) {
			let txn = db.conn.begin().await?;

			{
				use database_entities::mangas;
				let mut am: mangas::ActiveModel = manga.clone().into();
				let new_url = replace_host_preserve_path(&manga.url, &canonical_host)?;
				am.url = Set(new_url.clone());
				am.updated_at = Set(Utc::now().naive_utc());
				am.update(&txn).await?;
				manga.url = new_url;
			}

			{
				use database_entities::chapters;
				let chapter_models = chapters::Entity::find()
					.filter(chapters::Column::MangaId.eq(manga.id))
					.all(&txn)
					.await?;

				for ch in chapter_models {
					if let Ok(cu) = Url::parse(&ch.url) {
						if let Some(ch_host) = cu.host_str() {
							let ch_host_norm = ch_host.to_lowercase();
							if legacy_hosts.iter().any(|h| h == &ch_host_norm) {
								let new_ch_url = replace_host_preserve_path(&ch.url, &canonical_host)?;
								let mut cham: chapters::ActiveModel = ch.into();
								cham.url = Set(new_ch_url);
								cham.updated_at = Set(Utc::now().naive_utc());
								cham.update(&txn).await?;
							}
						}
					}
				}
			}

			txn.commit().await?;
		}
	}

	let scraped_manga = plugin
		.scrape_manga(manga.url.clone())
		.await
		.map_err(|e| SyncError::ScraperError { message: e.to_string() })?;

	let manga_created_at = manga.created_at.clone();
	let mut manga: database_entities::mangas::ActiveModel = manga.into();
	let parsed_date = scraped_manga.parse_release_date();

	manga.title = Set(scraped_manga.title);
	manga.img_url = Set(scraped_manga.img_url);
	manga.description = Set(Some(scraped_manga.description));
	manga.alternative_names = Set(Some(scraped_manga.alternative_names.join(", ")));
	manga.authors = Set(Some(scraped_manga.authors.join(", ")));
	manga.artists = Set(scraped_manga.artists.map(|artists| artists.join(", ")));
	manga.status = Set(Some(scraped_manga.status));
	manga.manga_type = Set(scraped_manga.manga_type);
	manga.release_date = Set(parsed_date);
	manga.genres = Set(Some(scraped_manga.genres.join(", ")));
	manga.updated_at = Set(Utc::now().naive_utc());

	if manga_created_at.is_none() {
		manga.created_at = Set(Some(Utc::now().naive_utc()));
	}

	let manga = manga.update(&db.conn).await?;

	let mut active_models: Vec<database_entities::chapters::ActiveModel> = Vec::new();
	let chapter_urls: Vec<String> = scraped_manga.chapters.iter().map(|c| c.url.clone()).collect();

	let existing_chapters: Vec<database_entities::chapters::Model> = database_entities::chapters::Entity::find()
		.filter(database_entities::chapters::Column::Url.is_in(chapter_urls.clone()))
		.all(&db.conn)
		.await?;

	let existing_urls: std::collections::HashSet<String> = existing_chapters.into_iter().map(|c| c.url).collect();

	for chapter in scraped_manga.chapters {
		if !existing_urls.contains(&chapter.url) {
			let new_chapter = database_entities::chapters::ActiveModel {
				manga_id: Set(manga.id),
				title: Set(chapter.title),
				url: Set(chapter.url),
				scanlation_group: Set(chapter.scanlation_group),
				created_at: Set(Utc::now().naive_utc()),
				updated_at: Set(Utc::now().naive_utc()),
				..Default::default()
			};

			active_models.push(new_chapter);
		}
	}

	if !active_models.is_empty() {
		database_entities::chapters::Entity::insert_many(active_models)
			.on_conflict(
				OnConflict::column(database_entities::chapters::Column::Url)
					.do_nothing()
					.to_owned(),
			)
			.exec(&db.conn)
			.await?;
	}

	Ok(())
}

fn host_from_base(base_url: &str) -> Option<String> {
	Url::parse(base_url).ok().and_then(|u| u.host_str().map(|s| s.to_string()))
}

fn replace_host_preserve_path(old_url: &str, new_host: &str) -> Result<String, SyncError> {
	let mut u = Url::parse(old_url)?;

	if let Some(idx) = new_host.find(':') {
		let host = &new_host[..idx];
		let port = &new_host[idx + 1..];

		u.set_host(Some(host)).map_err(|e| SyncError::UrlSetHost {
			host: host.to_string(),
			source: e,
		})?;

		let port_num: u16 = port.parse().map_err(|e| SyncError::InvalidPort {
			port: port.to_string(),
			source: e,
		})?;

		let _ = u.set_port(Some(port_num));
	} else {
		u.set_host(Some(new_host)).map_err(|e| SyncError::UrlSetHost {
			host: new_host.to_string(),
			source: e,
		})?;
		let _ = u.set_port(None);
	}

	Ok(u.into())
}
