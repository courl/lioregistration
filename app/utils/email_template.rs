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

/// A filled-in custom field value; its kind decides how it is inserted at `{{?key}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum CustomValue {
    Text(String),
    /// Markdown: rendered to HTML in emails, plain text paragraphs in attachments.
    Markdown(String),
    /// ISO `YYYY-MM-DD`, validated when resolved: inserted unescaped (it can't inject
    /// anything) so Typst can parse it inside a string literal.
    Date(String),
}

#[derive(Serialize, Deserialize)]
pub struct GlobalContext {
    pub edition: String,
    pub round: Option<String>,
    pub round_starts_at: NaiveDate,
    pub round_ends_at: NaiveDate,
    pub students: Option<Vec<(String, f32)>>,
    /// Values of the template's custom fields (headline, body, ...), by key.
    #[serde(default)]
    pub custom: BTreeMap<String, CustomValue>,
}

#[derive(Serialize, Deserialize, Default)]
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

/// Where a placeholder value ends up, which decides how it is escaped.
#[derive(Clone, Copy)]
enum Target {
    Html,
    Typst,
    /// A plain-text header line (the subject): inserted as is, collapsed to one line after.
    Line,
}

impl Target {
    fn escape(self, value: &str) -> String {
        match self {
            Self::Html => html_escape(value),
            Self::Typst => typst_escape(value),
            Self::Line => value.to_owned(),
        }
    }
}

impl CustomValue {
    fn render(&self, target: Target) -> String {
        match (self, target) {
            (Self::Date(value), _) => value.clone(),
            (Self::Markdown(value), Target::Html) => markdown_to_email_html(value),
            // Attachments get plain text: paragraphs survive, Markdown syntax shows as written.
            (Self::Markdown(value), Target::Typst) => value
                .split("\n\n")
                .map(|paragraph| typst_escape(paragraph.trim()))
                .filter(|paragraph| !paragraph.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n"),
            (Self::Text(value) | Self::Markdown(value), _) => target.escape(value),
        }
    }
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

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
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
    pub fn from_template(template: &str, ctx: &GlobalContext) -> Result<Self, EmailRenderError> {
        let filled_mjml = apply_global_ctx(template, ctx).replace(
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
        apply_instance_ctx_html(self.html.as_str(), ctx)
    }

    pub fn raw(&self) -> String {
        self.html.clone()
    }
}

fn apply_global_ctx_for(string: &str, ctx: &GlobalContext, target: Target) -> String {
    let filled = string
        .replace("{{!edition}}", &target.escape(&ctx.edition))
        .replace(
            "{{!round}}",
            &target.escape(ctx.round.as_deref().unwrap_or_default()),
        )
        .replace(
            "{{!round_starts_at}}",
            ctx.round_starts_at.to_string().as_str(),
        )
        .replace("{{!round_ends_at}}", ctx.round_ends_at.to_string().as_str());
    ctx.custom.iter().fold(filled, |acc, (key, value)| {
        acc.replace(&format!("{{{{?{key}}}}}"), &value.render(target))
    })
}

/// Fills global placeholders into an MJML email template, HTML-escaping the values.
pub fn apply_global_ctx(string: &str, ctx: &GlobalContext) -> String {
    apply_global_ctx_for(string, ctx, Target::Html)
}

/// Fills global placeholders into Typst attachment source, Typst-escaping the values.
pub fn apply_global_ctx_typst(string: &str, ctx: &GlobalContext) -> String {
    apply_global_ctx_for(string, ctx, Target::Typst)
}

/// Renders a template's subject line for one recipient: every placeholder may be used,
/// values are inserted as plain text on a single line.
pub fn render_subject(subject: &str, global: &GlobalContext, instance: &InstanceContext) -> String {
    single_line(&apply_instance_ctx_for(
        &apply_global_ctx_for(subject, global, Target::Line),
        instance,
        Target::Line,
    ))
}

/// Escapes text for a Markdown source: any ASCII punctuation may be backslash-escaped, which
/// also covers raw HTML and links. Line breaks become spaces, so no block can be started.
pub fn markdown_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in single_line(value).chars() {
        if c.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(c);
    }
    out
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

fn apply_instance_ctx_for(string: &str, ctx: &InstanceContext, target: Target) -> String {
    let escape = |value: &str| target.escape(value);
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
    apply_instance_ctx_for(string, ctx, Target::Html)
}

/// Fills per-recipient placeholders into Typst attachment source, Typst-escaping the values.
pub fn apply_instance_ctx_typst(string: &str, ctx: &InstanceContext) -> String {
    apply_instance_ctx_for(string, ctx, Target::Typst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::email_attachment::{EmailAttachmentContext, compile_typst_to_pdf};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn markdown_escape_neutralizes_links_and_html() {
        let html = markdown_to_email_html(&markdown_escape(
            "[x](https://evil.lu) <img src=a>\n\n# <https://evil.lu> D'Arc",
        ));
        assert!(!html.contains("<a"), "{html}");
        assert!(!html.contains("<img"), "{html}");
        assert!(!html.contains("<h1"), "{html}");
        assert!(html.contains("D'Arc"), "{html}");
    }

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

    fn global(custom: Vec<(&str, CustomValue)>) -> GlobalContext {
        GlobalContext {
            edition: "LIO 2026".into(),
            round: Some("Finals".into()),
            round_starts_at: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            round_ends_at: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            students: None,
            custom: custom.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }

    fn text(value: &str) -> CustomValue {
        CustomValue::Text(value.into())
    }

    #[test]
    fn custom_values_are_escaped_per_target() {
        let ctx = global(vec![
            ("place", text("<b>LGL</b> #x")),
            ("day", CustomValue::Date("2026-03-05".into())),
        ]);
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
    fn markdown_is_html_in_emails_and_plain_paragraphs_in_typst() {
        let ctx = global(vec![(
            "content",
            CustomValue::Markdown("Hello **Ada**,\n\nmeet at <b>#LGL</b>\nsoon.".into()),
        )]);
        let html = apply_global_ctx("{{?content}}", &ctx);
        assert!(html.contains("<strong>Ada</strong>"), "{html}");
        assert!(html.contains("<p>meet at"), "{html}");
        assert_eq!(
            apply_global_ctx_typst("{{?content}}", &ctx),
            "Hello \\*\\*Ada\\*\\*,\n\nmeet at \\<b\\>\\#LGL\\<\\/b\\> soon."
        );
    }

    #[test]
    fn subject_is_one_plain_line() {
        let global = global(vec![("headline", text("News <b>&</b>\nfor"))]);
        assert_eq!(
            render_subject(
                "{{?headline}} {{given_name}} – {{!edition}}",
                &global,
                &ctx("Ada")
            ),
            "News <b>&</b> for Ada – LIO 2026"
        );
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
