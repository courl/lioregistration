use crate::state::AppState;
use axum::Router;
use axum::routing::{delete, get, patch, post, put};

mod auth;
mod credentials;
mod edition;
mod emails;
mod registrations;
mod round;
mod schools;
mod student;

fn auth() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/me", get(auth::me))
}

fn editions() -> Router<AppState> {
    Router::new()
        .route("/", get(edition::editions))
        .route("/", post(edition::create_edition))
        .route("/active", get(edition::active_edition))
        .route("/{id}", get(edition::edition))
        .route("/{id}/activate", put(edition::set_active))
        .route("/{id}/statistics", get(edition::statistics))
        .route(
            "/{edition_id}/students",
            get(student::get_students_for_edition),
        )
        .route("/{edition_id}/rounds", get(round::rounds))
        .route("/{edition_id}/rounds", post(round::create_round))
        .route(
            "/{edition_id}/credentials",
            get(credentials::get_credentials),
        )
        .route(
            "/{edition_id}/credentials",
            post(credentials::create_credentials),
        )
        .route(
            "/{edition_id}/registrations",
            get(registrations::get_registrations),
        )
}

fn schools() -> Router<AppState> {
    Router::new()
        .route("/", get(schools::find_schools))
        .route("/{school_id}", get(schools::get_school))
        .route(
            "/{school_id}/merge/{other_school_id}",
            post(schools::merge_schools),
        )
}

fn emails() -> Router<AppState> {
    Router::new()
        .route("/", get(emails::get_templates))
        .route("/", post(emails::create_template))
        .route("/{id}", get(emails::get_template))
        .route("/{id}", put(emails::update_template))
        .route("/{id}/preview", post(emails::preview_template))
        .route("/{id}/send", post(emails::send_email))
        .route("/{id}/attachment", post(emails::add_attachment))
        .route("/{id}/attachment", get(emails::get_attachments))
        .route(
            "/{id}/attachment/{attachment_id}",
            get(emails::get_attachment),
        )
        .route(
            "/{id}/attachment/{attachment_id}",
            patch(emails::update_attachment),
        )
        .route(
            "/{id}/attachment/{attachment_id}",
            delete(emails::delete_attachment),
        )
}

fn rounds() -> Router<AppState> {
    Router::new()
        .route("/{round_id}", get(round::round))
        .route("/{round_id}", delete(round::delete_round))
        .route("/{round_id}", patch(round::update_round))
        .route(
            "/{round_id}/participations/{student_id}",
            patch(round::update_participation_details),
        )
        .route("/{round_id}/scores", post(round::bulk_update_scores))
        .route("/{round_id}/add", post(round::add_students))
        .route("/{round_id}/tasks", post(round::add_task))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth())
        .nest("/editions", editions())
        .nest("/schools", schools())
        .nest("/emails", emails())
        .nest("/rounds", rounds())
        // tasks
        .route("/tasks/{task_id}", patch(round::update_task))
        .route("/tasks/{task_id}", delete(round::delete_task))
        .route("/tasks/{task_id}/scores", get(round::get_task_scores))
        // credentials
        .route(
            "/credentials/{id}/unassign",
            post(credentials::unassign_credentials),
        )
        .route("/credentials/{id}", delete(credentials::delete_credentials))
        .route(
            "/credentials/assign/{registration_id}",
            post(credentials::assign_free_credentials),
        )
        // registrations
        .route(
            "/registrations/{registration_id}",
            get(registrations::get_registration),
        )
        .route(
            "/registrations/{registration_id}/candidates",
            get(registrations::get_candidates),
        )
        .route(
            "/registrations/{registration_id}/candidates/{student_id}",
            get(registrations::get_candidate),
        )
        .route(
            "/registrations/{registration_id}/link",
            post(registrations::validate_and_create_new_student),
        )
        .route(
            "/registrations/{registration_id}/link/{student_id}",
            post(registrations::link_to_student),
        )
        .route(
            "/registrations/{registration_id}/link",
            delete(registrations::unlink_from_student),
        )
        .route(
            "/registrations/{registration_id}/refuse",
            delete(registrations::refuse),
        )
        //
        .route("/students", get(student::get_students))
        .route("/students/{id}", get(student::get_student_details))
}
