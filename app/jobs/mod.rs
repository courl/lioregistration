use crate::error::{AppError, AppResult};
use crate::jobs::send_email::SendEmailPayload;
use sea_orm::{ActiveModelTrait, ConnectionTrait};

pub mod send_email;
pub mod worker;

pub enum Job {
    SendEmail(SendEmailPayload),
}

impl Job {
    pub(crate) fn serialize(&self) -> AppResult<(super::entity::job::JobType, String)> {
        let (job_type, payload) = match self {
            Job::SendEmail(send_email) => (
                super::entity::job::JobType::SendEmail,
                serde_json::to_string(send_email),
            ),
        };

        let payload = payload
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Cannot serialize result: {}", e)))?;
        Ok((job_type, payload))
    }

    pub(crate) fn deserialize(
        job_type: &super::entity::job::JobType,
        payload: &str,
    ) -> AppResult<Self> {
        Ok(match job_type {
            super::entity::job::JobType::SendEmail => {
                Self::SendEmail(serde_json::from_str(payload).map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Cannot deserialize payload: {}", e))
                })?)
            }
        })
    }
}

pub async fn enqueue(job: Job, db: &impl ConnectionTrait) -> AppResult<()> {
    let (job_type, payload) = job.serialize()?;

    super::entity::job::ActiveModel::create(job_type, payload)
        .insert(db)
        .await?;

    Ok(())
}
