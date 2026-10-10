#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "school")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(has_many)]
    pub registrations: HasMany<super::registration::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// See [`name_key`].
    #[must_use]
    pub fn key(&self) -> String {
        name_key(&self.name)
    }
}

impl ActiveModel {
    /// Stores the name normalized (see [`normalize_name`]).
    #[must_use]
    pub fn create(name: &str) -> Self {
        Self {
            name: ActiveValue::Set(normalize_name(name)),
            ..Default::default()
        }
    }
}

/// Display form of a school name: trimmed, inner whitespace collapsed to single spaces.
#[must_use]
pub fn normalize_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Identity of a school name: two names with the same key are the same school
/// ("Lycée A" == "  lycée   a"). Unicode-aware, unlike SQLite's `LOWER`.
#[must_use]
pub fn name_key(name: &str) -> String {
    normalize_name(name).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_whitespace_and_case() {
        assert_eq!(normalize_name("  Lycée   A \t"), "Lycée A");
        assert_eq!(name_key(" LYCÉE  a"), name_key("Lycée A"));
        assert_ne!(name_key("Lycée A"), name_key("Lycée B"));
    }
}
