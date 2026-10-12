#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::utils::email_template::CustomValue;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum TemplateType {
    Registration,
    Round,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CustomFieldKind {
    Text,
    Multiline,
    Date,
}

/// A value the sender fills in when using the template, inserted at `{{?key}}`. Every field
/// is required; `default` pre-fills it (and fills it for automatic emails).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub label: String,
    pub kind: CustomFieldKind,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, FromJsonQueryResult)]
pub struct CustomFields(pub Vec<CustomField>);

impl CustomFields {
    /// Keys must be unique `snake_case` identifiers, so `{{?key}}` can address them.
    pub fn validate(&self) -> AppResult<()> {
        let mut seen = HashSet::new();
        for field in &self.0 {
            let mut chars = field.key.chars();
            let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase())
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if !valid || !seen.insert(field.key.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Invalid or duplicate field key '{}': use lowercase letters, digits and '_'",
                    field.key
                )));
            }
        }
        Ok(())
    }

    /// Types the submitted values by the declared kinds. Every field needs a value (or a
    /// default); dates must be `YYYY-MM-DD`; unknown keys are refused.
    pub fn resolve_values(
        &self,
        mut values: BTreeMap<String, String>,
    ) -> AppResult<BTreeMap<String, CustomValue>> {
        let mut resolved = BTreeMap::new();
        for field in &self.0 {
            let value = values
                .remove(&field.key)
                .or_else(|| field.default.clone())
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    AppError::BadRequest(format!("Field '{}' is required", field.label))
                })?;
            let value = match field.kind {
                CustomFieldKind::Text => CustomValue::Text(value),
                // Multi-line fields are always Markdown.
                CustomFieldKind::Multiline => CustomValue::Markdown(value),
                CustomFieldKind::Date => {
                    NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| {
                        AppError::BadRequest(format!(
                            "Field '{}' must be a date (YYYY-MM-DD)",
                            field.label
                        ))
                    })?;
                    CustomValue::Date(value)
                }
            };
            resolved.insert(field.key.clone(), value);
        }
        if let Some(key) = values.keys().next() {
            return Err(AppError::BadRequest(format!(
                "Unknown field '{key}' for this template"
            )));
        }
        Ok(resolved)
    }
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub template: String,
    pub template_type: Option<TemplateType>,
    /// Subject line, may use any placeholder (`{{?headline}}`, `{{!edition}}`, ...).
    /// Nullable so schema-sync can add the column; `None`/empty falls back to the name.
    pub subject: Option<String>,
    /// Nullable so schema-sync can add the column to existing rows; `None` = no fields.
    #[sea_orm(column_type = "Json")]
    pub custom_fields: Option<CustomFields>,

    #[sea_orm(has_many)]
    pub attachments: HasMany<super::email_attachment_template::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(name: String, template: String, template_type: TemplateType) -> Self {
        Self {
            id: Default::default(),
            name: ActiveValue::Set(name),
            template: ActiveValue::Set(template),
            template_type: ActiveValue::Set(Some(template_type)),
            subject: ActiveValue::Set(None),
            custom_fields: ActiveValue::Set(None),
        }
    }
}

impl Model {
    #[must_use]
    pub fn custom_fields(&self) -> CustomFields {
        self.custom_fields.clone().unwrap_or_default()
    }

    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, kind: CustomFieldKind, default: Option<&str>) -> CustomField {
        CustomField {
            key: key.into(),
            label: key.to_uppercase(),
            kind,
            default: default.map(Into::into),
        }
    }

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect()
    }

    #[test]
    fn keys_are_validated() {
        let text = |key| field(key, CustomFieldKind::Text, None);
        assert!(CustomFields(vec![text("start_time")]).validate().is_ok());
        for bad in [
            vec![text("Start")],
            vec![text("a b")],
            vec![text("x"), text("x")],
        ] {
            assert!(matches!(
                CustomFields(bad).validate(),
                Err(AppError::BadRequest(_))
            ));
        }
    }

    #[test]
    fn every_field_needs_a_value() {
        let fields = CustomFields(vec![
            field("place", CustomFieldKind::Text, None),
            field("time", CustomFieldKind::Text, Some("13:00")),
            field("day", CustomFieldKind::Date, None),
            field("note", CustomFieldKind::Multiline, None),
        ]);
        let resolved = fields
            .resolve_values(values(&[
                ("place", " LGL "),
                ("day", "2026-03-05"),
                ("note", "*a*"),
            ]))
            .unwrap();
        assert_eq!(
            resolved,
            BTreeMap::from([
                ("place".into(), CustomValue::Text("LGL".into())),
                ("time".into(), CustomValue::Text("13:00".into())),
                ("day".into(), CustomValue::Date("2026-03-05".into())),
                ("note".into(), CustomValue::Markdown("*a*".into())),
            ])
        );

        let ok = [("place", "LGL"), ("day", "2026-03-05"), ("note", "n")];
        for bad in [
            values(&ok[1..]),
            values(&[ok[0], ("day", "5 March"), ok[2]]),
            values(&[ok[0], ok[1], ("note", "  ")]),
            values(&[ok[0], ok[1], ok[2], ("other", "x")]),
        ] {
            assert!(matches!(
                fields.resolve_values(bad),
                Err(AppError::BadRequest(_))
            ));
        }
    }
}
