use crate::entity::round_participation::Award;
use crate::entity::{registration, round, round_participation, student};
use crate::error::{AppError, AppResult};
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct AwardInfo {
    round: String,
    award: Award,
    count: u32,
}

pub async fn load_awards(
    ids: Vec<i32>,
    db: &impl ConnectionTrait,
) -> AppResult<HashMap<i32, Vec<AwardInfo>>> {
    let res = round_participation::Entity::find()
        .filter(round_participation::COLUMN.student_id.is_in(ids))
        .filter(round_participation::COLUMN.award.is_not_null())
        .find_also_related(round::Entity)
        .all(db)
        .await?;

    let mut grouped: HashMap<i32, HashMap<String, HashMap<Award, u32>>> = HashMap::new();

    for (participation, round) in res {
        let round = round.ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("participation missing related round"))
        })?;
        let award = participation
            .award
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("participation missing award")))?;

        *grouped
            .entry(participation.student_id)
            .or_default()
            .entry(round.name)
            .or_default()
            .entry(award)
            .or_default() += 1;
    }

    Ok(grouped
        .into_iter()
        .map(|(student_id, round_map)| {
            (
                student_id,
                round_map
                    .into_iter()
                    .flat_map(|(round_name, award_map)| {
                        award_map
                            .into_iter()
                            .map(|(award, count)| AwardInfo {
                                round: round_name.clone(),
                                award,
                                count,
                            })
                            .collect::<Vec<AwardInfo>>()
                    })
                    .collect(),
            )
        })
        .collect())
}

pub async fn load_awards_for_student(
    student: &student::Model,
    db: &impl ConnectionTrait,
) -> AppResult<Vec<AwardInfo>> {
    let mut awards = load_awards(vec![student.id], db).await?;
    Ok(awards.remove(&student.id).unwrap_or_else(Vec::new))
}

pub async fn list_students(
    db: &impl ConnectionTrait,
) -> AppResult<Vec<(student::Model, Vec<AwardInfo>)>> {
    let students = student::Entity::find().all(db).await?;
    let mut awards = load_awards(students.iter().map(|s| s.id).collect(), db).await?;

    Ok(students
        .into_iter()
        .map(|s| {
            let id = s.id;
            (s, awards.remove(&id).unwrap_or_else(Vec::new))
        })
        .collect())
}
pub async fn find_students(
    query: &str,
    db: &impl ConnectionTrait,
) -> AppResult<Vec<(student::Model, Vec<AwardInfo>)>> {
    if query.len() < 2 {
        return list_students(db).await;
    }
    let mut students = list_students(db)
        .await?
        .into_iter()
        .filter_map(|(student, awards)| {
            let similarity = (strsim::jaro_winkler(query, student.given_name.as_str())
                + strsim::jaro_winkler(query, student.family_name.as_str()))
                / 2.;
            if similarity > 0.5 {
                Some((similarity, (student, awards)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // todo: sort by ascending? and remove rev?
    students.sort_by(|(sim1, _), (sim2, _)| {
        sim1.partial_cmp(sim2).unwrap_or(Ordering::Less) // todo: here just random ordering::less, is there a better choice?
    });

    Ok(students.into_iter().rev().map(|(_, data)| data).collect())
}

/*
 * Returns student_id -> registration
 */
pub async fn load_registrations(
    edition_id: i32,
    students: Vec<&student::Model>,
    db: &impl ConnectionTrait,
) -> AppResult<HashMap<i32, registration::Model>> {
    let registrations = registration::Entity::find()
        .filter(
            registration::COLUMN
                .student_id
                .is_in(students.into_iter().map(|x| x.id)),
        )
        .filter(registration::COLUMN.edition_id.eq(edition_id))
        .all(db)
        .await?;

    registrations
        .into_iter()
        .map(|registration| {
            let student_id = registration.student_id.ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Database Corruption: registration id={} has no student_id",
                    registration.id
                ))
            })?;
            Ok((student_id, registration))
        })
        .collect::<AppResult<HashMap<_, _>>>()
}
