use crate::cli::admin::{add_admin, change_admin, list_admin};
use crate::cli::server::run_server;
use crate::settings::Settings;
use crate::state::AppState;
use clap::{Parser, Subcommand};

mod admin;
mod server;
mod table;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {},
    AddAdmin {
        #[arg(long, help = "The name for the user")]
        name: String,
        #[arg(long, help = "The email for the user")]
        email: String,
        #[arg(long, help = "The password for the user")]
        password: String,
        #[arg(long, help = "Give write access to the user")]
        write_access: bool,
    },
    ListAdmin {},
    ChangeAdmin {
        email: String,
        #[arg(long, help = "The new name for the user")]
        new_name: Option<String>,
        #[arg(long, help = "The new email for the user")]
        new_email: Option<String>,
        #[arg(long, help = "The new password for the user")]
        new_password: Option<String>,
        #[arg(long, help = "Give write access to the user")]
        write_access: Option<bool>,
        #[arg(long, help = "Delete this admin user")]
        delete: bool,
    },
}

pub async fn run_command(settings: Settings, state: AppState) -> anyhow::Result<()> {
    match Args::parse().command {
        Commands::Server {} => run_server(state, &settings.host, settings.port).await,
        Commands::AddAdmin {
            name,
            email,
            password,
            write_access,
        } => add_admin(state, &name, &email, &password, write_access).await,
        Commands::ListAdmin {} => list_admin(state).await,
        Commands::ChangeAdmin {
            email,
            new_name,
            new_email,
            new_password,
            write_access,
            delete,
        } => {
            change_admin(
                state,
                &email,
                new_name,
                new_email,
                new_password,
                write_access,
                delete,
            )
            .await
        }
    }
}
