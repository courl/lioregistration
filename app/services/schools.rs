use crate::entity::{registration, school};
use crate::error::{AppError, AppResult};
use crate::utils::search::rank_by_similarity;
use sea_orm::TransactionTrait;
use sea_orm::entity::prelude::*;

pub async fn list_schools(db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    Ok(school::Entity::find().all(db).await?)
}

/// Returns the school whose name matches `name` up to case and whitespace, creating it if
/// none does. Compared in Rust (the table is small) because SQLite's `LOWER` is ASCII-only.
pub async fn find_or_create(name: &str, db: &impl ConnectionTrait) -> AppResult<school::Model> {
    let key = school::name_key(name);
    match list_schools(db)
        .await?
        .into_iter()
        .find(|school| school.key() == key)
    {
        Some(school) => Ok(school),
        None => Ok(school::ActiveModel::create(name).insert(db).await?),
    }
}

pub async fn find_schools(query: &str, db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    let query = school::name_key(query);
    if query.chars().count() < 2 {
        return list_schools(db).await;
    }
    Ok(rank_by_similarity(list_schools(db).await?, |school| {
        strsim::jaro_winkler(&query, &school.key())
    }))
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
    use crate::utils::test_utils::tests::{
        create_edition, create_registration, create_school, date, setup_test,
    };

    #[tokio::test]
    async fn merge_reparents_registrations_and_deletes_source() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let keep = create_school(db, "Lycée A").await;
        let drop = create_school(db, "Lycée B").await;
        let reg = create_registration(db, &edition, &drop, None).await;

        merge(&keep, &drop, db).await.unwrap();

        let schools = list_schools(db).await.unwrap();
        assert_eq!(
            schools.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![keep.id]
        );
        let reg = registration::Entity::find_by_id(reg.id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reg.school_id, keep.id);
    }

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
