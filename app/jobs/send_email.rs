use std::sync::Arc;

use crate::entity::email_template;
use crate::error::{AppError, AppResult};
use crate::jobs::worker::{JobResult, WorkerSettings};
use crate::services::email_attachment::EmailAttachmentContext;
use crate::utils::email_template::{EmailTemplate, GlobalContext, InstanceContext};
use mail_send::mail_builder::MessageBuilder;
use mail_send::{SmtpClient, SmtpClientBuilder};
use sea_orm::{ConnectionTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Serialize, Deserialize)]
pub struct SendEmailPayload {
    pub template_id: i32,
    pub global_context: GlobalContext,
    pub body_markdown: String,
    pub instances: Option<Vec<(String, InstanceContext)>>,
}

pub(crate) struct SendEmailProcessor {
    sender_email: String,
    admin_email: String,
    smtp_server: String,
    smtp_host: u16,
    smtp_implicit_tls: bool,
    smtp_credentials: Option<(String, String)>,
    email_attachment_ctx: Arc<EmailAttachmentContext>,
}

impl SendEmailProcessor {
    pub(crate) fn create(settings: &WorkerSettings) -> AppResult<Self> {
        let email_attachment_ctx = Arc::new(EmailAttachmentContext::create(
            settings.assets_root.clone(),
        )?);
        Ok(Self {
            sender_email: settings.sender_email.clone(),
            admin_email: settings.admin_email.clone(),
            smtp_server: settings.smtp_server.clone(),
            smtp_host: settings.smtp_host,
            smtp_implicit_tls: settings.smtp_implicit_tls,
            smtp_credentials: settings.smtp_credentials.clone(),
            email_attachment_ctx,
        })
    }

    pub(crate) async fn run(
        &self,
        payload: SendEmailPayload,
        db: &impl ConnectionTrait,
    ) -> AppResult<JobResult> {
        let template = email_template::Entity::find_by_id(payload.template_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Cannot find template with id={}",
                    payload.template_id
                ))
            })?;
        let template = EmailTemplate::from_template(
            template.template.as_str(),
            &payload.global_context,
            &payload.body_markdown,
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Unable to render email: {}", e)))?;

        let mut client = {
            if let Some(credentials) = &self.smtp_credentials {
                Some(
                    SmtpClientBuilder::new(self.smtp_server.clone(), self.smtp_host)
                        .map_err(|_| {
                            AppError::Internal(anyhow::anyhow!("Cannot create smtp client"))
                        })?
                        .implicit_tls(self.smtp_implicit_tls)
                        .credentials(credentials.clone())
                        .connect()
                        .await
                        .map_err(|_| {
                            AppError::Internal(anyhow::anyhow!("Cannot create smtp client"))
                        })?,
                )
            } else {
                None
            }
        };

        self.send_emails(&mut client, template, payload).await
    }

    async fn send_emails<T>(
        &self,
        client: &mut Option<SmtpClient<T>>,
        template: EmailTemplate,
        payload: SendEmailPayload,
    ) -> AppResult<JobResult>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut errors = Vec::new();
        let mut partially_complete = false;
        if let Some(instances) = payload.instances {
            for (email, ctx) in instances {
                let email_content = template.instantiate(&ctx);
                if let Err(error) = self
                    .send_one_email(
                        client,
                        email_content,
                        (ctx.given_name + " " + ctx.family_name.as_ref(), email),
                        payload.global_context.headline.clone(),
                    )
                    .await
                {
                    errors.push(error);
                } else {
                    partially_complete = true;
                }
            }
        } else {
            self.send_one_email(
                client,
                template.raw(),
                ("Admin".into(), self.admin_email.clone()),
                "LIO Alert".into(),
            )
            .await?;
        };
        if partially_complete {
            Ok(JobResult::PartiallyComplete(errors))
        } else if !errors.is_empty() {
            // todo: we loose information here
            Err(errors.swap_remove(0))
        } else {
            Ok(JobResult::Complete)
        }
    }

    async fn send_one_email<T>(
        &self,
        client: &mut Option<SmtpClient<T>>,
        email_content: String,
        to: (String, String),
        subject: String,
    ) -> AppResult<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        if let Some(client) = client {
            let message = MessageBuilder::new()
                .from(self.sender_email.clone())
                .to(vec![(to.0.clone(), to.1.clone())])
                .subject(subject.clone())
                .html_body(email_content);

            if let Err(error) = client.send(message).await {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Cannot send email from {} to {} ({}) with subject '{}': {}",
                    self.sender_email.clone(),
                    to.0.clone(),
                    to.1.clone(),
                    subject.clone(),
                    error
                )));
            }
        } else {
            tracing::info!(
                "Sending email from {} to {} ({}) with content\n{}\n\n",
                self.sender_email,
                to.0,
                to.1,
                email_content
            );
        };
        Ok(())
    }
}
