use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use crate::entity::{email_attachment_template, email_template};
use crate::error::{AppError, AppResult};
use crate::jobs::worker::{JobResult, WorkerSettings};
use crate::services::email_attachment::{EmailAttachmentContext, compile_typst_to_pdf};
use crate::settings::SmtpSettings;
use crate::utils::email_template::{
    EmailTemplate, GlobalContext, InstanceContext, apply_global_ctx, apply_instance_ctx,
};
use mail_send::mail_builder::MessageBuilder;
use mail_send::{SmtpClient, SmtpClientBuilder};
use sea_orm::{ConnectionTrait, EntityTrait, ModelTrait};
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
    smtp: Option<SmtpSettings>,
    admin_email: String,
    email_attachment_ctx: Arc<EmailAttachmentContext>,
}

impl SendEmailProcessor {
    pub(crate) fn create(
        settings: &WorkerSettings,
        email_attachment_ctx: Arc<EmailAttachmentContext>,
    ) -> Self {
        Self {
            smtp: settings.smtp.clone(),
            admin_email: settings.admin_email.clone(),
            email_attachment_ctx,
        }
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
        let attachments = template
            .find_related(email_attachment_template::Entity)
            .all(db)
            .await?;

        let template = EmailTemplate::from_template(
            template.template.as_str(),
            &payload.global_context,
            &payload.body_markdown,
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Unable to render email: {}", e)))?;

        let mut client = {
            if let Some(smtp) = &self.smtp {
                Some(
                    SmtpClientBuilder::new(smtp.host.clone(), smtp.port)
                        .map_err(|_| {
                            AppError::Internal(anyhow::anyhow!("Cannot create smtp client"))
                        })?
                        .implicit_tls(smtp.implicit_tls)
                        .credentials((smtp.username.clone(), smtp.password.clone()))
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

        let attachments = attachments
            .into_iter()
            .map(|attachment| {
                (
                    attachment.name,
                    apply_global_ctx(attachment.template.as_str(), &payload.global_context),
                )
            })
            .collect();

        self.send_emails(&mut client, template, payload, attachments)
            .await
    }

    async fn send_emails<T>(
        &self,
        client: &mut Option<SmtpClient<T>>,
        template: EmailTemplate,
        payload: SendEmailPayload,
        attachments: Vec<(String, String)>,
    ) -> AppResult<JobResult>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut errors = Vec::new();
        let mut partially_complete = false;
        if let Some(instances) = payload.instances {
            for (email, ctx) in instances {
                let email_content = template.instantiate(&ctx);
                let attachments = attachments
                    .iter()
                    .map(|(name, content)| {
                        Ok((
                            "application/pdf",
                            name.as_str(),
                            compile_typst_to_pdf(
                                apply_instance_ctx(content.as_str(), &ctx).as_str(),
                                &self.email_attachment_ctx,
                            )?,
                        ))
                    })
                    .collect::<AppResult<Vec<_>>>()?;
                if let Err(error) = self
                    .send_one_email(
                        client,
                        email_content,
                        (ctx.given_name + " " + ctx.family_name.as_ref(), email),
                        payload.global_context.headline.clone(),
                        attachments,
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
                Vec::new(),
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
        attachments: Vec<(&str, &str, Vec<u8>)>,
    ) -> AppResult<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let sender_email = self
            .smtp
            .as_ref()
            .map_or(String::from("noreply@lio.lu"), |smtp| smtp.from.clone());
        if let Some(client) = client {
            let mut message = MessageBuilder::new()
                .from(sender_email.clone())
                .to(vec![(to.0.clone(), to.1.clone())])
                .subject(subject.clone())
                .html_body(email_content);

            for attachment in attachments {
                message = message.attachment(attachment.0, attachment.1, attachment.2);
            }

            if let Err(error) = client.send(message).await {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Cannot send email from {} to {} ({}) with subject '{}': {}",
                    sender_email.clone(),
                    to.0.clone(),
                    to.1.clone(),
                    subject.clone(),
                    error
                )));
            }
        } else {
            let base = String::from("/tmp/lioregistration/lastmail");
            let _ = std::fs::create_dir_all(&base);
            tracing::info!(
                "Sending email from {} to {} ({}) in {}",
                sender_email.clone(),
                to.0,
                to.1,
                &base
            );
            if let Ok(mut email) = File::create(base.clone() + "/mail.html") {
                let _ = email.write_all(&email_content.into_bytes());
            }

            for attachment in attachments {
                if let Ok(mut file) = File::create(base.clone() + "/" + attachment.1 + ".pdf") {
                    let _ = file.write_all(&attachment.2);
                }
            }
        };
        Ok(())
    }
}
