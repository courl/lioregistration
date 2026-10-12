#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

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
    /// An ISO date (`YYYY-MM-DD`), inserted unescaped so Typst can parse it.
    Date,
}

/// A value the sender fills in when using the template, inserted at `{{?key}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub label: String,
    pub kind: CustomFieldKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, FromJsonQueryResult)]
pub struct CustomFields(pub Vec<CustomField>);

fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl CustomFields {
    /// Keys must be unique `snake_case` identifiers, labels non-empty, and defaults valid.
    pub fn validate(&self) -> AppResult<()> {
        let mut seen = HashSet::new();
        for field in &self.0 {
            if !is_valid_key(&field.key) {
                return Err(AppError::BadRequest(format!(
                    "Invalid field key '{}': use lowercase letters, digits and '_', starting with a letter",
                    field.key
                )));
            }
            if !seen.insert(field.key.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Duplicate field key '{}'",
                    field.key
                )));
            }
            if field.label.trim().is_empty() {
                return Err(AppError::BadRequest(format!(
                    "Field '{}' needs a label",
                    field.key
                )));
            }
            if let Some(default) = field.default.as_deref() {
                field.check_value(default)?;
            }
        }
        Ok(())
    }

    /// Checks submitted values against the declaration: unknown keys and missing required
    /// values are refused, empty optional ones fall back to the default (or stay empty).
    pub fn resolve_values(
        &self,
        mut values: BTreeMap<String, String>,
    ) -> AppResult<BTreeMap<String, String>> {
        let mut resolved = BTreeMap::new();
        for field in &self.0 {
            let value = values
                .remove(&field.key)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
                .or_else(|| field.default.clone().filter(|v| !v.is_empty()));
            match value {
                Some(value) => {
                    field.check_value(&value)?;
                    resolved.insert(field.key.clone(), value);
                }
                None if field.required => {
                    return Err(AppError::BadRequest(format!(
                        "Field '{}' is required",
                        field.label
                    )));
                }
                None => {
                    resolved.insert(field.key.clone(), String::new());
                }
            }
        }
        if let Some(key) = values.keys().next() {
            return Err(AppError::BadRequest(format!(
                "Unknown field '{key}' for this template"
            )));
        }
        Ok(resolved)
    }
}

impl CustomField {
    fn check_value(&self, value: &str) -> AppResult<()> {
        if self.kind == CustomFieldKind::Date
            && NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err()
        {
            return Err(AppError::BadRequest(format!(
                "Field '{}' must be a date (YYYY-MM-DD)",
                self.label
            )));
        }
        Ok(())
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
            custom_fields: ActiveValue::Set(None),
        }
    }
}

impl Model {
    #[must_use]
    pub fn custom_fields(&self) -> CustomFields {
        self.custom_fields.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(
        key: &str,
        kind: CustomFieldKind,
        required: bool,
        default: Option<&str>,
    ) -> CustomField {
        CustomField {
            key: key.into(),
            label: key.to_uppercase(),
            kind,
            required,
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
    fn declaration_is_validated() {
        let ok = CustomFields(vec![field("start_time", CustomFieldKind::Text, true, None)]);
        assert!(ok.validate().is_ok());
        for bad in [
            vec![field("Start", CustomFieldKind::Text, false, None)],
            vec![field("a b", CustomFieldKind::Text, false, None)],
            vec![
                field("x", CustomFieldKind::Text, false, None),
                field("x", CustomFieldKind::Date, false, None),
            ],
            vec![field("d", CustomFieldKind::Date, false, Some("soon"))],
        ] {
            assert!(matches!(
                CustomFields(bad).validate(),
                Err(AppError::BadRequest(_))
            ));
        }
    }

    #[test]
    fn values_are_resolved_against_the_declaration() {
        let fields = CustomFields(vec![
            field("place", CustomFieldKind::Text, true, None),
            field("time", CustomFieldKind::Text, false, Some("13:00")),
            field("day", CustomFieldKind::Date, false, None),
            field("note", CustomFieldKind::Multiline, false, None),
        ]);
        let resolved = fields
            .resolve_values(values(&[
                ("place", " LGL "),
                ("time", ""),
                ("day", "2026-03-05"),
            ]))
            .unwrap();
        assert_eq!(
            resolved,
            values(&[
                ("place", "LGL"),
                ("time", "13:00"),
                ("day", "2026-03-05"),
                ("note", "")
            ])
        );

        for bad in [
            values(&[]),
            values(&[("place", "LGL"), ("day", "5 March")]),
            values(&[("place", "LGL"), ("other", "x")]),
        ] {
            assert!(matches!(
                fields.resolve_values(bad),
                Err(AppError::BadRequest(_))
            ));
        }
    }
}
