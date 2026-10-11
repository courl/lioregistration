use crate::entity;
use crate::entity::job::JobState;
use crate::error::{AppError, AppResult};
use crate::jobs::Job;
use crate::jobs::send_email::SendEmailProcessor;
use crate::services::email_attachment::EmailAttachmentContext;
use crate::settings::SmtpSettings;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, QueryFilter};
use sea_orm::{ColumnTrait, QueryOrder};
use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

pub(crate) enum JobResult {
    Complete,
    PartiallyComplete(Vec<AppError>),
}

pub struct WorkerSettings {
    pub smtp: Option<SmtpSettings>,
    pub admin_email: String,
}

pub struct Worker {
    send_email_processor: SendEmailProcessor,
    db: DatabaseConnection,
}

impl Worker {
    pub fn create(
        settings: &WorkerSettings,
        email_attachment_ctx: Arc<EmailAttachmentContext>,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            send_email_processor: SendEmailProcessor::create(settings, email_attachment_ctx),
            db,
        }
    }

    /// Processes jobs until `shutdown` flips to `true`. Shutdown is only observed between
    /// jobs, so a running job finishes instead of being left `Running` (and later `Failed`).
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) {
        if let Err(e) = self.fail_interrupted_jobs().await {
            tracing::error!("Could not recover interrupted jobs: {e:#}");
        }
        while !*shutdown.borrow() {
            let idle = match self.tick().await {
                Ok(true) => continue,
                Ok(false) => Duration::from_mins(1),
                Err(e) => {
                    tracing::error!("Job worker error: {e:#}");
                    Duration::from_secs(10)
                }
            };
            tokio::select! {
                () = tokio::time::sleep(idle) => {}
                _ = shutdown.changed() => {}
            }
        }
        tracing::info!("Job worker stopped");
    }

    /// A job still `Running` at startup was interrupted by a crash/restart. Retrying could
    /// re-send emails that already went out, so mark it `Failed` for an admin to look at.
    async fn fail_interrupted_jobs(&self) -> AppResult<()> {
        let result = entity::job::Entity::update_many()
            .col_expr(
                entity::job::Column::State,
                sea_orm::sea_query::Expr::value(JobState::Failed),
            )
            .filter(entity::job::COLUMN.state.eq(JobState::Running))
            .exec(&self.db)
            .await?;
        if result.rows_affected > 0 {
            tracing::error!(
                "Marked {} interrupted job(s) as failed; some of their emails may not have been sent",
                result.rows_affected
            );
        }
        Ok(())
    }

    async fn tick(&self) -> AppResult<bool> {
        let job = self
            .db
            .transaction::<_, _, AppError>(|txn| {
                Box::pin(async move {
                    let job = entity::job::Entity::find()
                        .filter(entity::job::COLUMN.state.eq(JobState::Pending))
                        .order_by_asc(entity::job::COLUMN.id)
                        .one(txn)
                        .await?;
                    if let Some(job) = job {
                        let mut job = job.into_active_model();
                        job.state = ActiveValue::Set(JobState::Running);
                        Ok(Some(job.update(txn).await?))
                    } else {
                        Ok(None)
                    }
                })
            })
            .await
            .map_err(AppError::from)?;

        let Some(job) = job else { return Ok(false) };

        let result = self.dispatch(&job).await;

        let job_id = job.id;
        let mut job = job.into_active_model();
        match result {
            Ok(result) => match result {
                JobResult::Complete => {
                    job.state = ActiveValue::Set(JobState::Completed);
                }
                JobResult::PartiallyComplete(errors) => {
                    job.state = ActiveValue::Set(JobState::PartiallyCompleted);
                    for error in errors {
                        tracing::error!("Job {} partially completed with error: {}", job_id, error);
                    }
                }
            },
            Err(e) => {
                tracing::error!("Job {} failed with error: {}", job_id, e);
                job.state = ActiveValue::Set(JobState::Failed);
            }
        }
        job.save(&self.db).await?;
        Ok(true)
    }

    async fn dispatch(&self, job: &entity::job::Model) -> AppResult<JobResult> {
        match Job::deserialize(&job.job_type, job.payload.as_str())? {
            Job::SendEmail(payload) => self.send_email_processor.run(payload, &self.db).await,
        }
    }
}
