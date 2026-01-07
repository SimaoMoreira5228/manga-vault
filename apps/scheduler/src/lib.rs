use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use database_connection::Database;
use database_entities::{favorite_mangas, mangas};
use queue::queue_item::QueueItem;
use queue::{EnqueueStrategy, TaskQueue};
use scraper_core::ScraperManager;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct MangaUpdateScheduler {
	queue: Arc<TaskQueue<MangaUpdateJob>>,
	db: Arc<Database>,
	interval: Duration,
	scraper_manager: Arc<ScraperManager>,
	scraper_cooldown: Arc<Mutex<ScraperCooldownTracker>>,
}

#[derive(Clone, Debug)]
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
		db: Arc<Database>,
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

					let result = manga_sync::sync_manga_with_scraper(
						db.as_ref(),
						scraper_manager.as_ref(),
						item.payload.manga_id,
						&item.payload.scraper_name,
					)
					.await;

					match result {
						Ok(()) => Ok(()),
						Err(err) => {
							if matches!(err, manga_sync::SyncError::ScraperNotFound { .. }) {
								tracing::warn!("Scraper plugin '{}' not found, skipping update", item.payload.scraper_name);
								ignored_scrapers.lock().await.insert(item.payload.scraper_name.clone());
								Ok(())
							} else if let manga_sync::SyncError::ScraperError(ref se) = err {
								if se.retryable {
									tracing::warn!(
										key = %item.key,
										scraper = %item.payload.scraper_name,
										"Retryable scraper error: {}. Attempt {}/{}",
										se,
										item.fail_count + 1,
										3
									);
									let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(err);
									Err(boxed)
								} else {
									tracing::error!(
										key = %item.key,
										scraper = %item.payload.scraper_name,
										"Permanent scraper error: {}. Skipping update.",
										se
									);
									Ok(())
								}
							} else {
								tracing::error!(
									key = %item.key,
									manga_id = item.payload.manga_id,
									scraper = %item.payload.scraper_name,
									attempt = item.fail_count + 1,
									"Manga sync job failed: {:#}",
									err
								);
								let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(err);
								Err(boxed)
							}
						}
					}
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

	pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
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
			.all(&self.db.conn)
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
			.all(&self.db.conn)
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

					let key = format!("manga-update-{}-{}", scraper, manga.id);
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
