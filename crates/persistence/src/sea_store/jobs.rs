use chrono::{DateTime, Utc};
use sea_orm::{
	ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

use super::{utc_to_db, *};
use crate::StoreResult;
use crate::entities::jobs;
use crate::queue::QueuedJob;
use crate::repo::{JobKind, JobRepository, JobRow, JobStatus};

fn row_from_model(model: &jobs::Model) -> JobRow {
	JobRow {
		id: model.id,
		kind: model.kind.clone(),
		subject: model.subject.clone(),
		attempts: model.attempts,
	}
}

fn open_filter() -> sea_orm::sea_query::SimpleExpr {
	jobs::Column::Status.is_in([
		JobStatus::Pending.as_str(),
		JobStatus::Running.as_str(),
		JobStatus::Retrying.as_str(),
	])
}

#[async_trait::async_trait]
impl JobRepository for SeaStore {
	async fn enqueue(&self, kind: JobKind, subject: &str, next_attempt_at: DateTime<Utc>) -> StoreResult<bool> {
		let already_open = jobs::Entity::find()
			.filter(jobs::Column::Kind.eq(kind.as_str()))
			.filter(jobs::Column::Subject.eq(subject))
			.filter(open_filter())
			.count(&self.db)
			.await?
			> 0;
		if already_open {
			return Ok(false);
		}
		let now = Utc::now();
		let id = uuid::Uuid::now_v7();
		jobs::ActiveModel {
			id: sea_orm::Set(id),
			kind: sea_orm::Set(kind.as_str().to_owned()),
			subject: sea_orm::Set(subject.to_owned()),
			status: sea_orm::Set(JobStatus::Pending.as_str().to_owned()),
			attempts: sea_orm::Set(0),
			next_attempt_at: sea_orm::Set(utc_to_db(next_attempt_at)),
			last_error: sea_orm::Set(None),
			created_at: sea_orm::Set(utc_to_db(now)),
			updated_at: sea_orm::Set(utc_to_db(now)),
		}
		.insert(&self.db)
		.await?;
		if let Some(queue) = &self.queue {
			let dispatched = QueuedJob {
				id,
				kind: kind.as_str().to_owned(),
				subject: subject.to_owned(),
				attempts: 0,
			};
			if let Err(error) = queue.dispatch(&dispatched, next_attempt_at).await {
				jobs::Entity::delete_by_id(id).exec(&self.db).await?;
				return Err(error);
			}
		}
		Ok(true)
	}

	async fn claim_due(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>> {
		if let Some(queue) = &self.queue {
			return self.claim_from_redis(queue, now, limit).await;
		}
		if self.db.get_database_backend() == sea_orm::DbBackend::Postgres {
			return self.claim_skip_locked(now, limit).await;
		}
		self.claim_mark(now, limit).await
	}

	async fn complete(&self, job_id: uuid::Uuid) -> StoreResult<()> {
		let row = jobs::Entity::find_by_id(job_id).one(&self.db).await?;
		jobs::Entity::update_many()
			.col_expr(
				jobs::Column::Status,
				sea_orm::sea_query::Expr::value(JobStatus::Done.as_str()),
			)
			.col_expr(
				jobs::Column::UpdatedAt,
				sea_orm::sea_query::Expr::value(utc_to_db(Utc::now())),
			)
			.filter(jobs::Column::Id.eq(job_id))
			.exec(&self.db)
			.await?;
		if let (Some(queue), Some(row)) = (&self.queue, row) {
			queue
				.settle(
					&QueuedJob {
						id: row.id,
						kind: row.kind,
						subject: row.subject,
						attempts: row.attempts,
					},
					None,
				)
				.await?;
		}
		Ok(())
	}

	async fn fail(&self, job_row: JobRow, error: &str, retry_at: Option<DateTime<Utc>>) -> StoreResult<()> {
		let status = if retry_at.is_some() {
			JobStatus::Retrying
		} else {
			JobStatus::Dead
		};
		if let Some(model) = jobs::Entity::find_by_id(job_row.id).one(&self.db).await? {
			let mut active: jobs::ActiveModel = model.into();
			active.status = sea_orm::Set(status.as_str().to_owned());
			active.attempts = sea_orm::Set(job_row.attempts + 1);
			active.last_error = sea_orm::Set(Some(error.to_owned()));
			active.next_attempt_at = sea_orm::Set(utc_to_db(retry_at.unwrap_or_else(Utc::now)));
			active.updated_at = sea_orm::Set(utc_to_db(Utc::now()));
			active.update(&self.db).await?;
		}
		if let Some(queue) = &self.queue {
			let progressed = QueuedJob {
				id: job_row.id,
				kind: job_row.kind,
				subject: job_row.subject,
				attempts: job_row.attempts + 1,
			};
			match retry_at {
				Some(at) => queue.reschedule(&progressed, at).await?,
				None => queue.settle(&progressed, Some(error)).await?,
			}
		}
		Ok(())
	}
}

impl SeaStore {
	async fn claim_from_redis(
		&self,
		queue: &crate::queue::RedisDispatch,
		now: DateTime<Utc>,
		limit: u64,
	) -> StoreResult<Vec<JobRow>> {
		let mut claimed = Vec::new();
		for job in queue.pop_due(now, limit).await? {
			let Some(model) = jobs::Entity::find_by_id(job.id).one(&self.db).await? else {
				queue.settle(&job, Some("job row missing")).await?;
				continue;
			};
			if !matches!(model.status.as_str(), "pending" | "retrying") {
				queue.settle(&job, Some("job row already settled")).await?;
				continue;
			}
			let mut active: jobs::ActiveModel = model.clone().into();
			active.status = sea_orm::Set(JobStatus::Running.as_str().to_owned());
			active.updated_at = sea_orm::Set(utc_to_db(now));
			active.update(&self.db).await?;
			claimed.push(row_from_model(&model));
		}
		Ok(claimed)
	}

	async fn claim_skip_locked(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>> {
		let txn = self.db.begin().await?;
		let due = jobs::Entity::find()
			.filter(jobs::Column::Status.is_in([JobStatus::Pending.as_str(), JobStatus::Retrying.as_str()]))
			.filter(jobs::Column::NextAttemptAt.lte(now))
			.order_by_asc(jobs::Column::NextAttemptAt)
			.limit(limit)
			.lock_with_behavior(
				sea_orm::sea_query::LockType::Update,
				sea_orm::sea_query::LockBehavior::SkipLocked,
			)
			.all(&txn)
			.await?;
		let mut claimed = Vec::with_capacity(due.len());
		for model in due {
			let mut active: jobs::ActiveModel = model.clone().into();
			active.status = sea_orm::Set(JobStatus::Running.as_str().to_owned());
			active.updated_at = sea_orm::Set(utc_to_db(now));
			active.update(&txn).await?;
			claimed.push(row_from_model(&model));
		}
		txn.commit().await?;
		Ok(claimed)
	}

	async fn claim_mark(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>> {
		let due = jobs::Entity::find()
			.filter(jobs::Column::Status.is_in([JobStatus::Pending.as_str(), JobStatus::Retrying.as_str()]))
			.filter(jobs::Column::NextAttemptAt.lte(now))
			.order_by_asc(jobs::Column::NextAttemptAt)
			.limit(limit)
			.all(&self.db)
			.await?;

		let mut claimed = Vec::with_capacity(due.len());
		for model in due {
			let mut active: jobs::ActiveModel = model.clone().into();
			active.status = sea_orm::Set(JobStatus::Running.as_str().to_owned());
			active.updated_at = sea_orm::Set(utc_to_db(now));
			active.update(&self.db).await?;
			claimed.push(row_from_model(&model));
		}
		Ok(claimed)
	}
}
