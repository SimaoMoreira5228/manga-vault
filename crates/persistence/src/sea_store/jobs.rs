use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

use super::{utc_to_db, *};
use crate::StoreResult;
use crate::entities::jobs;
use crate::repo::{JobKind, JobRepository, JobRow, JobStatus};

fn row_from_model(model: &jobs::Model) -> JobRow {
	JobRow {
		id: model.id,
		kind: model.kind.clone(),
		subject: model.subject.clone(),
		attempts: model.attempts,
	}
}

#[async_trait::async_trait]
impl JobRepository for SeaStore {
	async fn enqueue(&self, kind: JobKind, subject: &str, next_attempt_at: DateTime<Utc>) -> StoreResult<bool> {
		let open_statuses = [
			JobStatus::Pending.as_str(),
			JobStatus::Running.as_str(),
			JobStatus::Retrying.as_str(),
		];
		let already_open = jobs::Entity::find()
			.filter(jobs::Column::Kind.eq(kind.as_str()))
			.filter(jobs::Column::Subject.eq(subject))
			.filter(jobs::Column::Status.is_in(open_statuses))
			.count(&self.db)
			.await? > 0;
		if already_open {
			return Ok(false);
		}
		let now = Utc::now();
		jobs::ActiveModel {
			id: sea_orm::Set(uuid::Uuid::now_v7()),
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
		Ok(true)
	}

	async fn claim_due(&self, now: DateTime<Utc>, limit: u64) -> StoreResult<Vec<JobRow>> {
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

	async fn complete(&self, job_id: uuid::Uuid) -> StoreResult<()> {
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
		Ok(())
	}
}
