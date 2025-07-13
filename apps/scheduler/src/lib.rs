use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{TimeDelta, Utc};
use queue::queue_item::QueueItem;
use queue::{EnqueueStrategy, TaskQueue};
use rand::seq::SliceRandom;
use scraper_core::ScraperManager;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
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
	pub fn new(
		db: DatabaseConnection,
		scraper_manager: Arc<ScraperManager>,
		max_concurrency: usize,
		cooldown_duration: Duration,
	) -> Self {
		let cooldown_tracker = Arc::new(Mutex::new(ScraperCooldownTracker::new(cooldown_duration)));

		let process_fn = Arc::new({
			let db = db.clone();
			let scraper_manager = Arc::clone(&scraper_manager);
			let cooldown_tracker = Arc::clone(&cooldown_tracker);

			move |item: QueueItem<MangaUpdateJob>| {
				let db = db.clone();
				let scraper_manager = scraper_manager.clone();
				let cooldown_tracker = cooldown_tracker.clone();

				Box::pin(async move {
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

						let plugin = scraper_manager
							.get_plugin(&item.payload.scraper_name)
							.await
							.ok_or_else(|| anyhow::anyhow!("Scraper plugin '{}' not found", item.payload.scraper_name))?;

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
			interval: Duration::from_secs(30 * 60),
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

		let mangas = database_entities::mangas::Entity::find()
			.filter(database_entities::mangas::Column::UpdatedAt.lt(threshold))
			.all(&self.db)
			.await?;

		tracing::info!("Found {} mangas to update", mangas.len());

		let mut scraper_groups: HashMap<String, Vec<database_entities::mangas::Model>> = HashMap::new();
		for manga in mangas {
			scraper_groups.entry(manga.scraper.clone()).or_default().push(manga);
		}

		let mut interleaved = Vec::new();
		let mut scrapers: Vec<String> = scraper_groups.keys().cloned().collect();
		scrapers.shuffle(&mut rand::rng());

		while !scraper_groups.is_empty() {
			for scraper in &scrapers {
				if let Some(group) = scraper_groups.get_mut(scraper) {
					if let Some(manga) = group.pop() {
						interleaved.push(manga);
					}
					if group.is_empty() {
						scraper_groups.remove(scraper);
					}
				}
			}
		}

		for manga in interleaved {
			let job = MangaUpdateJob {
				manga_id: manga.id,
				scraper_name: manga.scraper.clone(),
				last_attempt: None,
			};

			let key = format!("manga-update-{}", manga.id);
			self.queue.insert(key, job, 5).await;
		}

		Ok(())
	}
}
