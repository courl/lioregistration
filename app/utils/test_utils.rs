#[cfg(test)]
pub mod tests {
    use crate::entity::admin_user::UserRole;
    use crate::entity::email_template::TemplateType;
    use crate::entity::registration::{FirstContact, Gender, NewRegistration, ReviewStatus};
    use crate::entity::round_participation::ParticipationState;
    use crate::entity::{
        admin_user, credentials, edition, email_template, registration, round, round_participation,
        school, student, task, task_score,
    };
    use crate::services::email_attachment::EmailAttachmentContext;
    use crate::state::AppState;
    use crate::utils::rate_limiter::RateLimiter;
    use crate::utils::token_generator::LoginTokenGenerator;
    use chrono::NaiveDate;
    use sea_orm::ActiveValue::Set;
    use sea_orm::{ActiveModelTrait, Database, DatabaseConnection};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    pub struct TestApp {
        pub state: AppState,
    }

    /// A directory under the system temp dir, removed on drop so a failing assertion doesn't
    /// leave fixtures behind.
    pub struct TempDir(pub PathBuf);

    impl TempDir {
        pub fn new(prefix: &str) -> Self {
            // Tests run concurrently in one process, so the pid alone isn't unique
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Self(std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id())))
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub async fn setup_test() -> TestApp {
        // `try_init` so the many tests sharing this harness don't panic on the
        // second call to set the global tracing subscriber.
        let _ = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "lio=info,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .try_init();
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.get_schema_registry("lio::entity::*")
            .sync(&db)
            .await
            .unwrap();
        TestApp {
            state: AppState {
                db: db.clone(),
                rate_limiter: Arc::new(RateLimiter::new(300, Duration::from_secs(60))),
                strict_rate_limiter: Arc::new(RateLimiter::new(5, Duration::from_secs(3600))),
                trust_proxy: false,
                cross_site_frontend: false,
                token_generator: Arc::new(LoginTokenGenerator::from_secret("secret", db.clone())),
                email_attachment_ctx: Arc::new(
                    EmailAttachmentContext::create(PathBuf::from("assets")).unwrap(),
                ),
            },
        }
    }

    impl TestApp {
        pub async fn with_user(
            self,
            email: &str,
            password: &str,
            name: &str,
            role: UserRole,
        ) -> Self {
            let user = admin_user::ActiveModel::create(email, password, name, role).unwrap();
            user.insert(&self.state.db).await.unwrap();
            self
        }

        pub async fn with_admin(self, email: &str, password: &str) -> Self {
            self.with_user(email, password, "Admin", UserRole::Admin)
                .await
        }
    }

    // --- Seed helpers shared by the service tests ---------------------------------

    pub async fn create_edition(
        db: &DatabaseConnection,
        year: u32,
        active: bool,
        opens: NaiveDate,
        closes: NaiveDate,
    ) -> edition::Model {
        edition::ActiveModel {
            year: Set(year),
            name: Set(format!("Edition {year}")),
            active: Set(active),
            registration_opens_at: Set(opens),
            registration_closes_at: Set(closes),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
    }

    pub async fn create_school(db: &DatabaseConnection, name: &str) -> school::Model {
        school::ActiveModel::create(name).insert(db).await.unwrap()
    }

    pub async fn create_student(
        db: &DatabaseConnection,
        given_name: &str,
        family_name: &str,
        birthdate: NaiveDate,
    ) -> student::Model {
        student::ActiveModel {
            given_name: Set(given_name.into()),
            family_name: Set(family_name.into()),
            birthdate: Set(birthdate),
            email: Set(String::new()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
    }

    pub async fn create_round(
        db: &DatabaseConnection,
        edition_id: i32,
        name: &str,
        starts: NaiveDate,
        ends: NaiveDate,
    ) -> round::Model {
        round::ActiveModel {
            edition_id: Set(edition_id),
            name: Set(name.into()),
            starts_at: Set(starts),
            ends_at: Set(ends),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
    }

    pub async fn create_task(
        db: &DatabaseConnection,
        round_id: i32,
        name: &str,
        ordinal: i32,
        max_points: u32,
    ) -> task::Model {
        task::ActiveModel {
            round_id: Set(round_id),
            name: Set(name.into()),
            ordinal: Set(ordinal),
            max_points: Set(max_points),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
    }

    pub async fn create_credential(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
        edition: &edition::Model,
    ) -> credentials::Model {
        credentials::ActiveModel::create(username, password, edition)
            .insert(db)
            .await
            .unwrap()
    }

    pub async fn create_participation(
        db: &DatabaseConnection,
        round: &round::Model,
        student: &student::Model,
        state: ParticipationState,
    ) -> round_participation::Model {
        round_participation::ActiveModel::create(round, student, state)
            .insert(db)
            .await
            .unwrap()
    }

    /// A fixed date in March 2026, for tests that only need stable, ordered dates.
    #[must_use]
    pub fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, day).unwrap()
    }

    #[must_use]
    pub fn today() -> NaiveDate {
        chrono::Local::now().naive_local().date()
    }

    /// Insert a registration for `edition`/`school`, optionally already linked to a student.
    pub async fn create_registration(
        db: &DatabaseConnection,
        edition: &edition::Model,
        school: &school::Model,
        student_id: Option<i32>,
    ) -> registration::Model {
        let mut reg = sample_new_registration(&school.name).into_active_model(edition.id, school);
        if let Some(student_id) = student_id {
            reg.student_id = Set(Some(student_id));
            reg.review_status = Set(ReviewStatus::Validated);
        }
        reg.insert(db).await.unwrap()
    }

    pub async fn create_score(
        db: &DatabaseConnection,
        task: &task::Model,
        participation: &round_participation::Model,
        points: f32,
    ) -> task_score::Model {
        task_score::ActiveModel::create(task, participation, points)
            .insert(db)
            .await
            .unwrap()
    }

    /// A minimal valid MJML template of the given type, so email enqueueing can succeed.
    pub async fn create_email_template(
        db: &DatabaseConnection,
        template_type: TemplateType,
    ) -> email_template::Model {
        email_template::ActiveModel::create(
            format!("{template_type:?}"),
            "<mjml><mj-body><mj-section><mj-column><mj-text>{{!headline}} {{given_name}}</mj-text></mj-column></mj-section></mj-body></mjml>".into(),
            template_type,
        )
        .insert(db)
        .await
        .unwrap()
    }

    #[must_use]
    pub fn sample_new_registration(school_name: &str) -> NewRegistration {
        NewRegistration {
            given_name: "Ada".into(),
            family_name: "Lovelace".into(),
            email: "ada@example.com".into(),
            birthdate: NaiveDate::from_ymd_opt(2005, 12, 10).unwrap(),
            gender: Gender::Female,
            address_street: "1 Main St".into(),
            address_postal_code: "L-1234".into(),
            address_locality: "Luxembourg".into(),
            phone: "+352 123456".into(),
            grade: "12".into(),
            school_name: school_name.into(),
            programming_languages: vec![String::from("C/C++")],
            first_contact_source: FirstContact::Teachers,
            first_contact_other: None,
        }
    }
}
