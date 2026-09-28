use crate::entity;
use crate::entity::job::JobState;
use crate::error::{AppError, AppResult};
use crate::jobs::Job;
use crate::jobs::send_email::SendEmailProcessor;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, QueryFilter};
use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait};
use std::path::PathBuf;
use std::time::Duration;

pub(crate) enum JobResult {
    Complete,
    PartiallyComplete(Vec<AppError>),
}

pub struct WorkerSettings {
    pub sender_email: String,
    pub admin_email: String,
    pub smtp_server: String,
    pub smtp_host: u16,
    pub smtp_implicit_tls: bool,
    pub smtp_credentials: Option<(String, String)>,
    pub assets_root: PathBuf,
}

pub struct Worker {
    send_email_processor: SendEmailProcessor,
    db: DatabaseConnection,
}

impl Worker {
    pub fn create(settings: &WorkerSettings, db: DatabaseConnection) -> Self {
        Self {
            send_email_processor: SendEmailProcessor::create(settings).expect("REASON"),
            db,
        }
    }

    pub async fn run(&self) {
        loop {
            match self.tick().await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(Duration::from_mins(1)).await,
                Err(e) => {
                    tracing::error!("Job worker error: {e:#}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn tick(&self) -> AppResult<bool> {
        let job = self
            .db
            .transaction::<_, _, AppError>(|txn| {
                Box::pin(async move {
                    let job = entity::job::Entity::find()
                        .filter(entity::job::COLUMN.state.eq(JobState::Pending))
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
