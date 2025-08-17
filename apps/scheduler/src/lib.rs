use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use database_entities::{favorite_mangas, mangas};
use queue::queue_item::QueueItem;
use queue::{EnqueueStrategy, TaskQueue};
use scraper_core::ScraperManager;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct MangaUpdateScheduler {
	queue: Arc<TaskQueue<MangaUpdateJob>>,
	db: DatabaseConnection,
	interval: Duration,
	scraper_manager: Arc<ScraperManager>,
	scraper_cooldown: Arc<Mutex<ScraperCooldownTracker>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct MangaUpdateJob {
	manga_id: i32,
	scraper_name: String,
	last_attempt: Option<Instant>,
}

struct ScraperCooldownTracker {
	last_used: HashMap<String, Instant>,
	cooldown: Duration,
}

impl ScraperCooldownTracker {
	fn new(cooldown: Duration) -> Self {
		Self {
			last_used: HashMap::new(),
			cooldown,
		}
	}

	fn needs_cooldown(&self, scraper: &str) -> bool {
		self.last_used
			.get(scraper)
			.map(|last| last.elapsed() < self.cooldown)
			.unwrap_or(false)
	}

	fn mark_used(&mut self, scraper: &str) {
		self.last_used.insert(scraper.to_string(), Instant::now());
	}
}

impl MangaUpdateScheduler {
	/// Creates a new `MangaUpdateScheduler`.
	/// # Arguments
	/// * `db` - The database connection.
	/// * `scraper_manager` - The scraper manager to use for scraping manga.
	/// * `max_concurrency` - The maximum number of concurrent tasks.
	/// * `search_interval` - The interval at which to search for manga updates.
	/// * `cooldown_duration` - The cooldown duration for each scraper.
	pub fn new(
		db: DatabaseConnection,
		scraper_manager: Arc<ScraperManager>,
		max_concurrency: usize,
		search_interval: Duration,
		cooldown_duration: Duration,
	) -> Self {
		let cooldown_tracker = Arc::new(Mutex::new(ScraperCooldownTracker::new(cooldown_duration)));
		let ignored_scrapers = Arc::new(Mutex::new(HashSet::new()));

		let process_fn = Arc::new({
			let db = db.clone();
			let scraper_manager = Arc::clone(&scraper_manager);
			let cooldown_tracker = Arc::clone(&cooldown_tracker);
			let ignored_scrapers = Arc::clone(&ignored_scrapers);

			move |item: QueueItem<MangaUpdateJob>| {
				let db = db.clone();
				let scraper_manager = scraper_manager.clone();
				let cooldown_tracker = cooldown_tracker.clone();
				let ignored_scrapers = ignored_scrapers.clone();

				Box::pin(async move {
					{
						let ignored = ignored_scrapers.lock().await;
						if ignored.contains(&item.payload.scraper_name) {
							tracing::debug!("Skipping ignored scraper: {}", item.payload.scraper_name);
							return Ok(());
						}
					}

					let mut tracker = cooldown_tracker.lock().await;
					if tracker.needs_cooldown(&item.payload.scraper_name) {
						let delay = tracker.cooldown - tracker.last_used[&item.payload.scraper_name].elapsed();
						tokio::time::sleep(delay).await;
					}

					tracker.mark_used(&item.payload.scraper_name);
					drop(tracker);

					let result = (async {
						let manga = database_entities::mangas::Entity::find_by_id(item.payload.manga_id)
							.one(&db)
							.await
							.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?
							.ok_or_else(|| anyhow::anyhow!("Manga {} not found", item.payload.manga_id))?;

						let plugin = scraper_manager.get_plugin(&item.payload.scraper_name).await;

						if plugin.is_none() {
							tracing::warn!("Scraper plugin '{}' not found, skipping update", item.payload.scraper_name);
							ignored_scrapers.lock().await.insert(item.payload.scraper_name.clone());
							return Ok(());
						}

						let plugin = plugin.unwrap();

						let scraped_manga = plugin.scrape_manga(manga.url.clone()).await?;

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

						let manga = manga.update(&db).await.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;

						let mut active_models: Vec<database_entities::chapters::ActiveModel> = Vec::new();
						let chapter_urls: Vec<String> = scraped_manga.chapters.iter().map(|c| c.url.clone()).collect();

						let existing_chapters: Vec<database_entities::chapters::Model> =
							database_entities::chapters::Entity::find()
								.filter(database_entities::chapters::Column::MangaId.eq(manga.id.clone()))
								.filter(database_entities::chapters::Column::Url.is_in(chapter_urls.clone()))
								.all(&db)
								.await
								.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;

						let existing_urls: std::collections::HashSet<String> =
							existing_chapters.into_iter().map(|c| c.url).collect();

						for chapter in scraped_manga.chapters {
							if !existing_urls.contains(&chapter.url) {
								let new_chapter = database_entities::chapters::ActiveModel {
									manga_id: Set(manga.id),
									title: Set(chapter.title),
									url: Set(chapter.url),
									created_at: Set(Utc::now().naive_utc()),
									updated_at: Set(Utc::now().naive_utc()),
									..Default::default()
								};

								active_models.push(new_chapter);
							}
						}

						if !active_models.is_empty() {
							database_entities::chapters::Entity::insert_many(active_models)
								.exec(&db)
								.await
								.map_err(|e: sea_orm::DbErr| anyhow::Error::from(e))?;
						}

						Ok(())
					})
					.await;

					result
				})
					as Pin<
						Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
					>
			}
		});

		let queue = Arc::new(TaskQueue::new(
			process_fn,
			100,
			3,
			100,
			max_concurrency,
			EnqueueStrategy::BestEffort,
			Some(300),
		));

		Self {
			queue,
			db,
			interval: search_interval,
			scraper_manager,
			scraper_cooldown: cooldown_tracker,
		}
	}

	pub async fn start(self: Arc<Self>) {
		loop {
			if let Err(e) = self.schedule_updates().await {
				tracing::error!("Failed to schedule manga updates: {:#}", e);
			}

			tokio::time::sleep(self.interval).await;
		}
	}

	async fn schedule_updates(&self) -> Result<(), anyhow::Error> {
		tracing::info!("Scheduling manga updates...");
		let threshold = Utc::now() - TimeDelta::try_hours(1).unwrap();

		let critical_mangas = self.fetch_critical_mangas(threshold).await?;

		let mut scheduled = self.schedule_manga_batch(critical_mangas).await;

		if self.queue.len() < self.queue.max_size() {
			let remaining_mangas = self.fetch_remaining_mangas(threshold).await?;
			scheduled += self.schedule_manga_batch(remaining_mangas).await;
		}

		tracing::info!("Scheduled {} manga updates", scheduled);
		Ok(())
	}

	async fn fetch_critical_mangas(&self, threshold: DateTime<Utc>) -> Result<Vec<(mangas::Model, i64)>, anyhow::Error> {
		mangas::Entity::find()
			.filter(mangas::Column::UpdatedAt.lt(threshold))
			.order_by_desc(mangas::Column::UpdatedAt)
			.limit(500)
			.find_with_related(favorite_mangas::Entity)
			.all(&self.db)
			.await?
			.into_iter()
			.map(|(manga, favs)| {
				let fav_count = favs.len() as i64;
				Ok((manga, fav_count))
			})
			.collect()
	}

	async fn fetch_remaining_mangas(&self, threshold: DateTime<Utc>) -> Result<Vec<(mangas::Model, i64)>, anyhow::Error> {
		mangas::Entity::find()
			.filter(mangas::Column::UpdatedAt.lt(threshold))
			.find_with_related(favorite_mangas::Entity)
			.all(&self.db)
			.await?
			.into_iter()
			.map(|(manga, favs)| {
				let fav_count = favs.len() as i64;
				Ok((manga, fav_count))
			})
			.collect()
	}

	async fn schedule_manga_batch(&self, mangas: Vec<(mangas::Model, i64)>) -> usize {
		let mut scheduled = 0;
		let mut scraper_buckets: HashMap<String, Vec<_>> = HashMap::new();

		for (manga, fav_count) in mangas {
			scraper_buckets
				.entry(manga.scraper.clone())
				.or_default()
				.push((manga, fav_count));
		}

		while !scraper_buckets.is_empty() {
			for (scraper, mangas) in scraper_buckets.iter_mut() {
				if let Some((manga, fav_count)) = mangas.pop() {
					let priority = self.calculate_priority(fav_count, manga.updated_at);
					let job = MangaUpdateJob {
						manga_id: manga.id,
						scraper_name: scraper.clone(),
						last_attempt: None,
					};

					let key = format!("manga-update-{}", manga.id);
					if self.queue.insert(key, job, priority).await {
						scheduled += 1;
					}
				}
			}

			scraper_buckets.retain(|_, v| !v.is_empty());
		}
		scheduled
	}

	fn calculate_priority(&self, fav_count: i64, last_updated: NaiveDateTime) -> u8 {
		let base: u8 = match fav_count {
			0 => 1,
			1..=5 => 2,
			6..=20 => 3,
			21..=50 => 4,
			51..=100 => 5,
			101..=500 => 6,
			_ => 7,
		};

		let last_updated_utc = DateTime::from_naive_utc_and_offset(last_updated, Utc);
		let hours_stale = (Utc::now() - last_updated_utc).num_hours().max(0) as u8;

		base.saturating_add(hours_stale.min(10))
	}
}
