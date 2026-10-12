use chrono::NaiveDate;
use mrml::prelude::render::RenderOptions;
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    /// Values of the template's custom fields, filled into `{{?key}}` placeholders.
    #[serde(default)]
    pub custom: BTreeMap<String, String>,
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
                    html_escape(&student.0),
                    student.1
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
        // Custom fields go last so their placeholders also work inside the body.
        let filled_mjml = apply_custom_ctx_with(
            &apply_fixed_global_ctx_with(template, ctx, html_escape)
                .replace("{{!content}}", &content_html)
                .replace(
                    "{{!students}}",
                    generate_student_table(&ctx.students).as_str(),
                ),
            ctx,
            html_escape,
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
        apply_instance_ctx_html(self.html.as_str(), ctx)
    }

    pub fn raw(&self) -> String {
        self.html.clone()
    }
}

fn apply_fixed_global_ctx_with(
    string: &str,
    ctx: &GlobalContext,
    escape: impl Fn(&str) -> String,
) -> String {
    string
        .replace("{{!headline}}", &escape(&ctx.headline))
        .replace("{{!edition}}", &escape(&ctx.edition))
        .replace(
            "{{!round}}",
            &escape(ctx.round.as_deref().unwrap_or_default()),
        )
        .replace(
            "{{!round_starts_at}}",
            ctx.round_starts_at.to_string().as_str(),
        )
        .replace("{{!round_ends_at}}", ctx.round_ends_at.to_string().as_str())
}

/// An ISO date (`2026-03-05`) can't inject anything, so it is inserted unescaped: Typst
/// templates can then parse it inside a string literal, like `{{!round_starts_at}}`.
fn is_iso_date(value: &str) -> bool {
    value.len() == 10 && NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

fn apply_custom_ctx_with(
    string: &str,
    ctx: &GlobalContext,
    escape: impl Fn(&str) -> String,
) -> String {
    ctx.custom
        .iter()
        .fold(string.to_owned(), |acc, (key, value)| {
            let value = if is_iso_date(value) {
                value.clone()
            } else {
                escape(value)
            };
            acc.replace(&format!("{{{{?{key}}}}}"), &value)
        })
}

fn apply_global_ctx_with(
    string: &str,
    ctx: &GlobalContext,
    escape: impl Fn(&str) -> String,
) -> String {
    apply_custom_ctx_with(
        &apply_fixed_global_ctx_with(string, ctx, &escape),
        ctx,
        &escape,
    )
}

/// Fills global placeholders into an MJML email template, HTML-escaping the values.
pub fn apply_global_ctx(string: &str, ctx: &GlobalContext) -> String {
    apply_global_ctx_with(string, ctx, html_escape)
}

/// Fills global placeholders into Typst attachment source, Typst-escaping the values.
pub fn apply_global_ctx_typst(string: &str, ctx: &GlobalContext) -> String {
    apply_global_ctx_with(string, ctx, typst_escape)
}

/// Escapes text for HTML element content and attribute values.
pub fn html_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escapes text for Typst *markup* context: every character with markup or code meaning
/// (`#` starts code, `$` math, `*`/`_` emphasis, `/` comments, ...) is backslash-escaped, so
/// registrant-supplied values render literally and can never execute as Typst code.
pub fn typst_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if matches!(
            c,
            '\\' | '#'
                | '$'
                | '*'
                | '_'
                | '`'
                | '<'
                | '>'
                | '@'
                | '['
                | ']'
                | '='
                | '-'
                | '+'
                | '/'
                | '~'
                | '"'
                | '\''
        ) {
            out.push('\\');
        }
        if c == '\n' || c == '\r' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

fn apply_instance_ctx_with(
    string: &str,
    ctx: &InstanceContext,
    escape: impl Fn(&str) -> String,
) -> String {
    string
        .replace("{{given_name}}", &escape(&ctx.given_name))
        .replace("{{family_name}}", &escape(&ctx.family_name))
        .replace("{{school}}", &escape(&ctx.school))
        .replace("{{grade}}", &escape(&ctx.grade))
        .replace(
            "{{credentials_username}}",
            &escape(ctx.credentials_username.as_deref().unwrap_or_default()),
        )
        .replace(
            "{{credentials_password}}",
            &escape(ctx.credentials_password.as_deref().unwrap_or_default()),
        )
}

/// Fills per-recipient placeholders into rendered email HTML, HTML-escaping the values.
pub fn apply_instance_ctx_html(string: &str, ctx: &InstanceContext) -> String {
    apply_instance_ctx_with(string, ctx, html_escape)
}

/// Fills per-recipient placeholders into Typst attachment source, Typst-escaping the values.
pub fn apply_instance_ctx_typst(string: &str, ctx: &InstanceContext) -> String {
    apply_instance_ctx_with(string, ctx, typst_escape)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::email_attachment::{EmailAttachmentContext, compile_typst_to_pdf};
    use std::path::PathBuf;
    use std::sync::Arc;

    fn ctx(given_name: &str) -> InstanceContext {
        InstanceContext {
            given_name: given_name.into(),
            family_name: "Lovelace".into(),
            school: "Lycée A".into(),
            grade: "12".into(),
            credentials_username: Some("user01".into()),
            credentials_password: None,
        }
    }

    #[test]
    fn html_values_are_escaped() {
        let out = apply_instance_ctx_html(
            "<p>{{given_name}} {{family_name}} {{credentials_password}}</p>",
            &ctx("<img src=x onerror=\"alert('x')\">&"),
        );
        assert_eq!(
            out,
            "<p>&lt;img src=x onerror=&quot;alert(&#39;x&#39;)&quot;&gt;&amp; Lovelace </p>"
        );
    }

    fn global(custom: &[(&str, &str)]) -> GlobalContext {
        GlobalContext {
            headline: "News".into(),
            edition: "LIO 2026".into(),
            round: Some("Finals".into()),
            round_starts_at: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            round_ends_at: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            students: None,
            custom: custom
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn custom_values_are_escaped_per_target() {
        let ctx = global(&[("place", "<b>LGL</b> #x"), ("day", "2026-03-05")]);
        assert_eq!(
            apply_global_ctx("{{?place}} {{?day}} {{?unknown}}", &ctx),
            "&lt;b&gt;LGL&lt;/b&gt; #x 2026-03-05 {{?unknown}}"
        );
        assert_eq!(
            apply_global_ctx_typst("{{?place}} \"{{?day}}\"", &ctx),
            "\\<b\\>LGL\\<\\/b\\> \\#x \"2026-03-05\""
        );
    }

    #[test]
    fn custom_placeholders_work_in_the_body() {
        let ctx = global(&[("place", "Lycée <A>")]);
        let template = "<mjml><mj-body><mj-section><mj-column><mj-text>{{!content}}</mj-text></mj-column></mj-section></mj-body></mjml>";
        let email = EmailTemplate::from_template(template, &ctx, "Meet at {{?place}}").unwrap();
        assert!(email.raw().contains("Meet at Lycée &lt;A&gt;"));
    }

    #[test]
    fn only_real_dates_skip_escaping() {
        assert!(is_iso_date("2026-03-05"));
        assert!(!is_iso_date("2026-3-5"));
        assert!(!is_iso_date("2026-13-01"));
        assert!(!is_iso_date("2026-03-05#x"));
    }

    #[test]
    fn student_table_names_are_escaped() {
        let table = generate_student_table(&Some(vec![("<b>Eve</b>".into(), 1.0)]));
        assert!(table.contains("&lt;b&gt;Eve&lt;/b&gt;"));
        assert!(!table.contains("<b>Eve"));
    }

    #[test]
    fn typst_values_render_literally() {
        assert_eq!(typst_escape("#panic()"), "\\#panic()");
        assert_eq!(typst_escape("a*b_c $x$"), "a\\*b\\_c \\$x\\$");
        assert_eq!(typst_escape("Jean-Luc O'Neil"), "Jean\\-Luc O\\'Neil");
        assert_eq!(typst_escape("line\nbreak"), "line break");
    }

    #[test]
    fn typst_injection_does_not_execute() {
        let ctx_ = Arc::new(EmailAttachmentContext::create(PathBuf::from("assets")).unwrap());
        let template = "Hello {{given_name}} from {{school}}";
        // Unescaped, this would abort compilation via `panic`; escaped it is just text.
        let raw = template.replace("{{given_name}}", "#panic(\"pwned\")");
        assert!(
            compile_typst_to_pdf(&raw, &ctx_).is_err(),
            "control: raw input executes"
        );
        for name in [
            "#panic(\"pwned\")",
            "#read(\"/etc/passwd\")",
            "$ x $ // */ [ ]",
        ] {
            let source = apply_instance_ctx_typst(template, &ctx(name));
            compile_typst_to_pdf(&source, &ctx_).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
