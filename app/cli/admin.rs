use crate::cli::table::Table;
use crate::entity::admin_user;
use crate::entity::admin_user::UserRole;
use crate::state::AppState;
use anyhow::anyhow;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel, Order};

pub async fn add_admin(
    state: AppState,
    name: &str,
    email: &str,
    password: &str,
    write_access: bool,
) -> anyhow::Result<()> {
    let user = admin_user::ActiveModel::create(
        email,
        password,
        name,
        if write_access {
            UserRole::Admin
        } else {
            UserRole::Observer
        },
    )?
    .insert(&state.db)
    .await?;
    Table::new(
        vec![
            String::new(),
            "Email".into(),
            "Password".into(),
            "Name".into(),
        ],
        Some(vec![1, 20, 20, 10]),
    )
    .add_row(vec![
        match user.role {
            UserRole::Admin => "x",
            UserRole::Observer => " ",
        }
        .into(),
        user.email,
        password.into(),
        user.name,
    ])
    .draw();
    Ok(())
}

pub async fn list_admin(state: AppState) -> anyhow::Result<()> {
    let mut table = Table::new(
        vec![String::new(), "Email".into(), "Name".into()],
        Some(vec![1, 20, 10]),
    );
    admin_user::Entity::find()
        .order_by_id(Order::Asc)
        .all(&state.db)
        .await?
        .iter()
        .for_each(|user| {
            table.add_row(vec![
                match user.role {
                    UserRole::Admin => "x",
                    UserRole::Observer => " ",
                }
                .into(),
                user.email.clone(),
                user.name.clone(),
            ]);
        });
    table.draw();
    Ok(())
}

pub async fn change_admin(
    state: AppState,
    email: &str,
    new_name: Option<String>,
    new_email: Option<String>,
    new_password: Option<String>,
    write_access: Option<bool>,
    delete: bool,
) -> anyhow::Result<()> {
    let user = admin_user::Entity::find_by_email(email)
        .one(&state.db)
        .await?;
    if let Some(user) = user {
        let mut user = user.into_active_model();
        if delete {
            user.delete(&state.db).await?;
            println!("User {email} has been deleted");
            return Ok(());
        }
        if let Some(new_name) = new_name {
            user.name = ActiveValue::Set(new_name);
        }
        if let Some(new_email) = new_email {
            user.email = ActiveValue::Set(new_email);
        }
        if let Some(new_password) = new_password {
            user.set_password(&new_password)?;
        }
        if let Some(write_access) = write_access {
            user.role = ActiveValue::Set(if write_access {
                UserRole::Admin
            } else {
                UserRole::Observer
            });
        }

        let user = user.update(&state.db).await?;
        Table::new(
            vec![String::new(), "Email".into(), "Name".into()],
            Some(vec![1, 20, 10]),
        )
        .add_row(vec![
            match user.role {
                UserRole::Admin => "x",
                UserRole::Observer => " ",
            }
            .into(),
            user.email,
            user.name,
        ])
        .draw();
        Ok(())
    } else {
        Err(anyhow!("User not found: {email}"))
    }
}
