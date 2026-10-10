use crate::entity::{registration, school};
use crate::error::{AppError, AppResult};
use sea_orm::{ActiveModelTrait, TransactionTrait};
use sea_orm::entity::prelude::*;

pub async fn list_schools(db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    Ok(school::Entity::find().all(db).await?)
}

/// Returns the school whose name matches `name` up to case and whitespace, creating it if
/// none does. Compared in Rust (the table is small) because SQLite's `LOWER` is ASCII-only.
pub async fn find_or_create(name: &str, db: &impl ConnectionTrait) -> AppResult<school::Model> {
    let key = school::name_key(name);
    if let Some(school) = list_schools(db)
        .await?
        .into_iter()
        .find(|school| school.key() == key)
    {
        return Ok(school);
    }
    Ok(school::ActiveModel::create(name).insert(db).await?)
}

pub async fn find_schools(query: &str, db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    let query = school::name_key(query);
    if query.chars().count() < 2 {
        return list_schools(db).await;
    }
    let mut schools = list_schools(db)
        .await?
        .into_iter()
        .filter_map(|school| {
            let similarity = strsim::jaro_winkler(&query, &school.key());
            if similarity > 0.5 {
                Some((similarity, school))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // Most similar first.
    schools.sort_by(|(sim1, _), (sim2, _)| sim2.total_cmp(sim1));

    Ok(schools.into_iter().map(|(_, school)| school).collect())
}

/// Merge `other` into `into`: re-point every registration from `other` onto `into`, then
/// delete `other`. Both steps run in one transaction so a half-merged state is impossible.
pub async fn merge(
    into: &school::Model,
    other: &school::Model,
    db: &DatabaseConnection,
) -> AppResult<()> {
    let into_id = into.id;
    let other_id = other.id;
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            registration::Entity::update_many()
                .filter(registration::Column::SchoolId.eq(other_id))
                .col_expr(registration::Column::SchoolId, Expr::value(into_id))
                .exec(txn)
                .await?;

            school::Entity::delete_by_id(other_id).exec(txn).await?;
            Ok(())
        })
    })
    .await
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::tests::{create_school, setup_test};

    #[tokio::test]
    async fn find_or_create_ignores_case_and_whitespace() {
        let app = setup_test().await;
        let db = &app.state.db;
        let existing = create_school(db, "Lycée A").await;

        let same = find_or_create("  lycée   a ", db).await.unwrap();
        assert_eq!(same.id, existing.id);

        let other = find_or_create(" Lycée  B ", db).await.unwrap();
        assert_ne!(other.id, existing.id);
        assert_eq!(other.name, "Lycée B");
        assert_eq!(list_schools(db).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn find_schools_is_case_insensitive() {
        let app = setup_test().await;
        let db = &app.state.db;
        create_school(db, "Lycée Classique").await;
        create_school(db, "Zzz").await;

        let found = find_schools("LYCÉE CLASSIQUE", db).await.unwrap();
        assert_eq!(found[0].name, "Lycée Classique");
        assert!(found.iter().all(|school| school.name != "Zzz"));
    }
}
