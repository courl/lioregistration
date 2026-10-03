use chrono::NaiveDate;
use mrml::prelude::render::RenderOptions;
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EmailRenderError {
    #[error("mjml parse failed: {0}")]
    MjmlParse(String),
    #[error("mjml render failed: {0}")]
    MjmlRender(String),
}

#[derive(Serialize, Deserialize)]
pub struct GlobalContext {
    pub headline: String,
    pub edition: String,
    pub round: Option<String>,
    pub round_starts_at: NaiveDate,
    pub round_ends_at: NaiveDate,
    pub students: Option<Vec<(String, f32)>>,
}

#[derive(Serialize, Deserialize)]
pub struct InstanceContext {
    pub given_name: String,
    pub family_name: String,
    pub school: String,
    pub grade: String,
    pub credentials_username: Option<String>,
    pub credentials_password: Option<String>,
}

pub struct EmailTemplate {
    html: String,
}

fn markdown_to_email_html(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(source, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn generate_student_table(students: &Option<Vec<(String, f32)>>) -> String {
    if let Some(students) = students {
        let mut value = String::from(
            r#"
                <mj-table>
                    <tr style="border-bottom:1px solid #ecedee;text-align:left;padding:15px 0;">
                      <th style="padding: 0 15px 0 0;">Student</th>
                      <th style="padding: 0 0 0 15px;">Score</th>
                    </tr>;
            "#,
        );

        for student in students {
            value.push_str(
                format!(
                    "
                <tr>
                  <td style=\"padding: 0 15px 0 0;\">{}</td>
                  <td style=\"padding: 0 0 0 15px;\">{:.2}</td>
                </tr>",
                    student.0, student.1
                )
                .as_str(),
            );
        }

        value.push_str("</mj-table>");
        value
    } else {
        String::new()
    }
}

impl EmailTemplate {
    pub fn from_template(
        template: &str,
        ctx: &GlobalContext,
        body_markdown: &str,
    ) -> Result<Self, EmailRenderError> {
        let content_html = markdown_to_email_html(body_markdown);
        let filled_mjml = apply_global_ctx(template, ctx)
            .replace("{{!content}}", &content_html)
            .replace(
                "{{!students}}",
                generate_student_table(&ctx.students).as_str(),
            );

        let parsed =
            mrml::parse(&filled_mjml).map_err(|e| EmailRenderError::MjmlParse(e.to_string()))?;

        let render_opts = RenderOptions::default();
        let html = parsed
            .element
            .render(&render_opts)
            .map_err(|e| EmailRenderError::MjmlRender(e.to_string()))?;
        Ok(Self { html })
    }

    pub fn instantiate(&self, ctx: &InstanceContext) -> String {
        apply_instance_ctx(self.html.as_str(), ctx)
    }

    pub fn raw(&self) -> String {
        self.html.clone()
    }
}

pub fn apply_global_ctx(string: &str, ctx: &GlobalContext) -> String {
    let empty_string = String::new();
    string
        .replace("{{!headline}}", &ctx.headline)
        .replace("{{!edition}}", &ctx.edition)
        .replace("{{!round}}", ctx.round.as_ref().unwrap_or(&empty_string))
        .replace(
            "{{!round_starts_at}}",
            ctx.round_starts_at.to_string().as_str(),
        )
        .replace("{{!round_ends_at}}", ctx.round_ends_at.to_string().as_str())
}

pub fn apply_instance_ctx(string: &str, ctx: &InstanceContext) -> String {
    let empty_string = String::new();
    string
        .replace("{{given_name}}", &ctx.given_name)
        .replace("{{family_name}}", &ctx.family_name)
        .replace("{{school}}", &ctx.school)
        .replace("{{grade}}", &ctx.grade)
        .replace(
            "{{credentials_username}}",
            ctx.credentials_username.as_ref().unwrap_or(&empty_string),
        )
        .replace(
            "{{credentials_password}}",
            ctx.credentials_password.as_ref().unwrap_or(&empty_string),
        )
}
