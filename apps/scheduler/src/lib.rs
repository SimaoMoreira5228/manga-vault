use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use database_connection::Database;
use database_entities::{favorite_mangas, favorite_novels, mangas, novels};
use queue::queue_item::QueueItem;
use queue::{EnqueueStrategy, TaskQueue};
use scraper_core::ScraperManager;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm::{DbBackend, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

#[allow(dead_code)]
pub struct MangaUpdateScheduler {
	queue: Arc<TaskQueue<UpdateJob>>,
	db: Arc<Database>,
	interval: Duration,
	scraper_manager: Arc<ScraperManager>,
	scraper_limiter: Arc<Mutex<PerScraperLimiter>>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum ItemType {
	Manga,
	Novel,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct UpdateJob {
	item_id: i32,
	scraper_name: String,
	item_type: ItemType,
	last_attempt: Option<Instant>,
}

struct PerScraperLimiter {
	semaphores: HashMap<String, Arc<Semaphore>>,
	last_used: HashMap<String, Instant>,
	cooldown: Duration,
	per_scraper_concurrency: usize,
	overrides: HashMap<String, usize>,
}

impl PerScraperLimiter {
	pub fn new(per_scraper_concurrency: usize, cooldown: Duration, overrides: HashMap<String, usize>) -> Self {
		Self {
			semaphores: HashMap::new(),
			last_used: HashMap::new(),
			cooldown,
			per_scraper_concurrency,
			overrides,
		}
	}

	fn get_semaphore(&mut self, scraper: &str) -> Arc<Semaphore> {
		let cap = self.overrides.get(scraper).cloned().unwrap_or(self.per_scraper_concurrency);
		self.semaphores
			.entry(scraper.to_string())
			.or_insert_with(|| Arc::new(Semaphore::new(cap)))
			.clone()
	}

	fn mark_used(&mut self, scraper: &str) {
		self.last_used.insert(scraper.to_string(), Instant::now());
	}

	fn needs_cooldown(&self, scraper: &str) -> Option<Duration> {
		self.last_used
			.get(scraper)
			.map(|last| {
				if last.elapsed() < self.cooldown {
					Some(self.cooldown - last.elapsed())
				} else {
					None
				}
			})
			.flatten()
	}
}

#[derive(Debug, Deserialize, Serialize, config_derive::Config)]
#[config(name = "scheduler")]
pub struct Config {
	#[serde(default)]
	pub queue_max_size: usize,
	#[serde(default)]
	pub channel_capacity: usize,
	#[serde(default)]
	pub max_concurrency: usize,
	#[serde(default)]
	pub default_per_scraper_concurrency: usize,
	#[serde(default)]
	pub per_scraper_limits: BTreeMap<String, usize>,
	#[serde(default)]
	pub queue_aging_interval_secs: Option<u64>,
	#[serde(default)]
	pub cooldown_seconds: u64,
	#[serde(default)]
	pub search_interval_seconds: u64,
	#[serde(default)]
	pub claim_limit: u64,
	#[serde(default)]
	pub enqueue_strategy: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			queue_max_size: 100,
			channel_capacity: 100,
			max_concurrency: 5,
			default_per_scraper_concurrency: 2,
			per_scraper_limits: BTreeMap::new(),
			queue_aging_interval_secs: Some(300),
			cooldown_seconds: 10,
			search_interval_seconds: 30 * 60,
			claim_limit: 500,
			enqueue_strategy: "best_effort".to_string(),
		}
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
		let cfg = Config::load();

		let per_scraper_overrides: HashMap<String, usize> = cfg.per_scraper_limits.into_iter().collect();

		let per_scraper_concurrency = if cfg.default_per_scraper_concurrency > 0 {
			cfg.default_per_scraper_concurrency
		} else {
			std::cmp::max(1, max_concurrency / 2)
		};

		let cooldown = if cfg.cooldown_seconds > 0 {
			Duration::from_secs(cfg.cooldown_seconds)
		} else {
			cooldown_duration
		};

		let scraper_limiter = Arc::new(Mutex::new(PerScraperLimiter::new(
			per_scraper_concurrency,
			cooldown,
			per_scraper_overrides,
		)));
		let ignored_scrapers = Arc::new(Mutex::new(HashSet::new()));

		let process_fn = Arc::new({
			let db = db.clone();
			let scraper_manager = Arc::clone(&scraper_manager);
			let scraper_limiter = Arc::clone(&scraper_limiter);
			let ignored_scrapers = Arc::clone(&ignored_scrapers);

			move |item: QueueItem<UpdateJob>| {
				let db = db.clone();
				let scraper_manager = scraper_manager.clone();
				let scraper_limiter = scraper_limiter.clone();
				let ignored_scrapers = ignored_scrapers.clone();

				Box::pin(async move {
					{
						let ignored = ignored_scrapers.lock().await;
						if ignored.contains(&item.payload.scraper_name) {
							tracing::debug!("Skipping ignored scraper: {}", item.payload.scraper_name);
							return Ok(());
						}
					}

					if let Some(delay) = {
						let lim = scraper_limiter.lock().await;
						lim.needs_cooldown(&item.payload.scraper_name)
					} {
						tokio::time::sleep(delay).await;
					}

					let sem: Arc<Semaphore> = {
						let mut lim = scraper_limiter.lock().await;
						let s = lim.get_semaphore(&item.payload.scraper_name);
						lim.mark_used(&item.payload.scraper_name);
						s
					};

					let _permit: OwnedSemaphorePermit = match sem.clone().acquire_owned().await {
						Ok(p) => p,
						Err(_) => {
							tracing::warn!("Semaphore closed for scraper {}", item.payload.scraper_name);
							return Ok(());
						}
					};

					let result = match item.payload.item_type {
						ItemType::Manga => {
							manga_sync::sync_manga_with_scraper(
								db.as_ref(),
								scraper_manager.as_ref(),
								item.payload.item_id,
								&item.payload.scraper_name,
							)
							.await
						}
						ItemType::Novel => {
							manga_sync::sync_novel_with_scraper(
								db.as_ref(),
								scraper_manager.as_ref(),
								item.payload.item_id,
								&item.payload.scraper_name,
							)
							.await
						}
					};

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
									item_id = item.payload.item_id,
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

		let enqueue_strategy = match cfg.enqueue_strategy.as_str() {
			"block" => EnqueueStrategy::Block,
			_ => EnqueueStrategy::BestEffort,
		};

		let queue = Arc::new(TaskQueue::new(
			process_fn,
			cfg.queue_max_size,
			3,
			cfg.channel_capacity,
			std::cmp::max(cfg.max_concurrency, max_concurrency),
			enqueue_strategy,
			cfg.queue_aging_interval_secs,
		));

		Self {
			queue,
			db,
			interval: search_interval,
			scraper_manager,
			scraper_limiter: scraper_limiter,
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

		let claim_limit = 500u64;
		let claimed_mangas = self.claim_mangas(threshold, claim_limit).await?;
		let mut scheduled = self.schedule_manga_batch(claimed_mangas).await;

		if self.queue.len() < self.queue.max_size() {
			let available = self.queue.max_size().saturating_sub(self.queue.len());
			let fetch_limit = (available.saturating_mul(3)).max(10) as u64;
			let remaining = self.claim_mangas(threshold, fetch_limit).await?;
			scheduled += self.schedule_manga_batch(remaining).await;
		}

		let claimed_novels = self.claim_novels(threshold, claim_limit).await?;
		scheduled += self.schedule_novel_batch(claimed_novels).await;

		if self.queue.len() < self.queue.max_size() {
			let available = self.queue.max_size().saturating_sub(self.queue.len());
			let fetch_limit = (available.saturating_mul(3)).max(10) as u64;
			let remaining = self.claim_novels(threshold, fetch_limit).await?;
			scheduled += self.schedule_novel_batch(remaining).await;
		}

		tracing::info!("Scheduled {} updates", scheduled);
		Ok(())
	}

	async fn claim_novels(&self, threshold: DateTime<Utc>, limit: u64) -> Result<Vec<(novels::Model, i64)>, anyhow::Error> {
		if self.db.db_type == "postgresql" {
			let ts = threshold.format("%Y-%m-%d %H:%M:%S").to_string();
			let sql = format!(
				"WITH c AS (SELECT id FROM novels WHERE updated_at < '{}' ORDER BY updated_at DESC LIMIT {limit} FOR UPDATE SKIP LOCKED) SELECT id FROM c",
				ts,
				limit = limit
			);

			let txn = self.db.conn.begin().await?;
			let stmt = Statement::from_string(DbBackend::Postgres, sql);
			let rows = txn.query_all(stmt).await?;

			let mut ids: Vec<i32> = Vec::new();
			for row in rows {
				let id: i32 = row.try_get("", "id")?;
				ids.push(id);
			}

			if ids.is_empty() {
				txn.commit().await?;
				return Ok(vec![]);
			}

			let models = novels::Entity::find()
				.filter(novels::Column::Id.is_in(ids.clone()))
				.all(&txn)
				.await?;

			let fav_rows = favorite_novels::Entity::find()
				.filter(favorite_novels::Column::NovelId.is_in(ids.clone()))
				.all(&txn)
				.await?;

			let mut fav_count: HashMap<i32, i64> = HashMap::new();
			for f in fav_rows {
				*fav_count.entry(f.novel_id).or_insert(0) += 1;
			}

			let mut model_map: HashMap<i32, novels::Model> = HashMap::new();
			for m in models {
				model_map.insert(m.id, m);
			}

			let mut result: Vec<(novels::Model, i64)> = Vec::new();
			for id in ids {
				if let Some(m) = model_map.remove(&id) {
					let cnt = *fav_count.get(&id).unwrap_or(&0);
					result.push((m, cnt));
				}
			}

			txn.commit().await?;
			return Ok(result);
		}

		let models = novels::Entity::find()
			.filter(novels::Column::UpdatedAt.lt(threshold))
			.order_by_desc(novels::Column::UpdatedAt)
			.limit(limit as u64)
			.all(&self.db.conn)
			.await?;

		let ids: Vec<i32> = models.iter().map(|m| m.id).collect();
		if ids.is_empty() {
			return Ok(vec![]);
		}

		let fav_rows = favorite_novels::Entity::find()
			.filter(favorite_novels::Column::NovelId.is_in(ids.clone()))
			.all(&self.db.conn)
			.await?;

		let mut fav_count: HashMap<i32, i64> = HashMap::new();
		for f in fav_rows {
			*fav_count.entry(f.novel_id).or_insert(0) += 1;
		}

		let mut model_map: HashMap<i32, novels::Model> = HashMap::new();
		for m in models {
			model_map.insert(m.id, m);
		}

		let mut result: Vec<(novels::Model, i64)> = Vec::new();
		for id in ids {
			if let Some(m) = model_map.remove(&id) {
				let cnt = *fav_count.get(&id).unwrap_or(&0);
				result.push((m, cnt));
			}
		}

		Ok(result)
	}

	async fn claim_mangas(&self, threshold: DateTime<Utc>, limit: u64) -> Result<Vec<(mangas::Model, i64)>, anyhow::Error> {
		if self.db.db_type == "postgresql" {
			let ts = threshold.format("%Y-%m-%d %H:%M:%S").to_string();
			let sql = format!(
				"WITH c AS (SELECT id FROM mangas WHERE updated_at < '{}' ORDER BY updated_at DESC LIMIT {limit} FOR UPDATE SKIP LOCKED) SELECT id FROM c",
				ts,
				limit = limit
			);

			let txn = self.db.conn.begin().await?;
			let stmt = Statement::from_string(DbBackend::Postgres, sql);
			let rows = txn.query_all(stmt).await?;

			let mut ids: Vec<i32> = Vec::new();
			for row in rows {
				let id: i32 = row.try_get("", "id")?;
				ids.push(id);
			}

			if ids.is_empty() {
				txn.commit().await?;
				return Ok(vec![]);
			}

			let models = mangas::Entity::find()
				.filter(mangas::Column::Id.is_in(ids.clone()))
				.all(&txn)
				.await?;

			let fav_rows = favorite_mangas::Entity::find()
				.filter(favorite_mangas::Column::MangaId.is_in(ids.clone()))
				.all(&txn)
				.await?;

			let mut fav_count: HashMap<i32, i64> = HashMap::new();
			for f in fav_rows {
				*fav_count.entry(f.manga_id).or_insert(0) += 1;
			}

			let mut model_map: HashMap<i32, mangas::Model> = HashMap::new();
			for m in models {
				model_map.insert(m.id, m);
			}

			let mut result: Vec<(mangas::Model, i64)> = Vec::new();
			for id in ids {
				if let Some(m) = model_map.remove(&id) {
					let cnt = *fav_count.get(&id).unwrap_or(&0);
					result.push((m, cnt));
				}
			}

			txn.commit().await?;
			return Ok(result);
		}

		let models = mangas::Entity::find()
			.filter(mangas::Column::UpdatedAt.lt(threshold))
			.order_by_desc(mangas::Column::UpdatedAt)
			.limit(limit as u64)
			.all(&self.db.conn)
			.await?;

		let ids: Vec<i32> = models.iter().map(|m| m.id).collect();
		if ids.is_empty() {
			return Ok(vec![]);
		}

		let fav_rows = favorite_mangas::Entity::find()
			.filter(favorite_mangas::Column::MangaId.is_in(ids.clone()))
			.all(&self.db.conn)
			.await?;

		let mut fav_count: HashMap<i32, i64> = HashMap::new();
		for f in fav_rows {
			*fav_count.entry(f.manga_id).or_insert(0) += 1;
		}

		let mut model_map: HashMap<i32, mangas::Model> = HashMap::new();
		for m in models {
			model_map.insert(m.id, m);
		}

		let mut result: Vec<(mangas::Model, i64)> = Vec::new();
		for id in ids {
			if let Some(m) = model_map.remove(&id) {
				let cnt = *fav_count.get(&id).unwrap_or(&0);
				result.push((m, cnt));
			}
		}

		Ok(result)
	}

	async fn schedule_manga_batch(&self, mangas: Vec<(mangas::Model, i64)>) -> usize {
		let mut scheduled = 0;

		for (manga, fav_count) in mangas.into_iter() {
			let scraper = manga.scraper.clone();
			let priority = self.calculate_priority(fav_count, manga.updated_at);
			let job = UpdateJob {
				item_id: manga.id,
				scraper_name: scraper.clone(),
				item_type: ItemType::Manga,
				last_attempt: None,
			};

			let key = format!("manga-update-{}-{}", scraper, manga.id);
			if self.queue.insert(key, job, priority).await {
				scheduled += 1;
			}
		}

		scheduled
	}

	async fn schedule_novel_batch(&self, novels_vec: Vec<(novels::Model, i64)>) -> usize {
		let mut scheduled = 0;

		for (novel, fav_count) in novels_vec.into_iter() {
			let scraper = novel.scraper.clone();
			let priority = self.calculate_priority(fav_count, novel.updated_at);
			let job = UpdateJob {
				item_id: novel.id,
				scraper_name: scraper.clone(),
				item_type: ItemType::Novel,
				last_attempt: None,
			};

			let key = format!("novel-update-{}-{}", scraper, novel.id);
			if self.queue.insert(key, job, priority).await {
				scheduled += 1;
			}
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
