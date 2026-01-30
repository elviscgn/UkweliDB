mod commands;
mod config;
mod ledger_manager;
mod user_store;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
// use ukweli_db::Workflow;

// TODO
// init – initialise database and genesis record #10 done
// record append – append signed records to the ledger #11 done
// record verify – verify hash chain and signatures #12 done
// workflow load – load JSON/YAML workflow definitions #13 done
// workflow list – list available workflows #14 done
// record show – inspect records by entity or range #15 done
// state current – compute current entity state by replay #16

#[derive(Parser)]
#[command(name = "ukweli")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        /// custom database path (default: ~/.ukweli/default.ukweli)
        #[arg(short, long)]
        db_path: Option<PathBuf>,
    },

    /// user management comms
    User {
        #[command(subcommand)]
        command: UserCommands,
    },
    #[command(subcommand)]
    Record(RecordCommands),
    #[command(subcommand)]
    Workflow(WorkflowCommands),
    // #[command(subcommand)]
    // State(StateCommands),
}

#[derive(Subcommand)]
enum UserCommands {
    Create { user_id: String },
    List,
    Delete { user_id: String },
    Show { user_id: String },
}

#[derive(Subcommand)]
enum RecordCommands {
    Append {
        payload: String,

        #[arg(short, long, value_delimiter = ',')]
        signers: Vec<String>,
    },
    Verify,
    Show {
        index: usize,
    },
    List {
        #[arg(long)]
        signer: Option<String>,

        #[arg(long)]
        from: Option<usize>,

        #[arg(long)]
        to: Option<usize>,

        #[arg(long)]
        limit: Option<usize>,
    },
    Compact,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    Load { file: PathBuf },
    List,
    Show { workflow_id: String },
    Delete { workflow_id: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { db_path } => {
            commands::init::run(db_path)?;
        }

        Commands::Record(command) => match command {
            RecordCommands::Append { payload, signers } => {
                commands::record::append(payload, signers)?;
            }
            RecordCommands::Verify => {
                commands::record::verify()?;
            }
            RecordCommands::Show { index } => {
                commands::record::show(index)?;
            }
            RecordCommands::List {
                signer,
                from,
                to,
                limit,
            } => {
                commands::record::list(signer, from, to, limit)?;
            }

            RecordCommands::Compact => {
                commands::record::compact()?;
            }
        },

        Commands::User { command } => match command {
            UserCommands::Create { user_id } => {
                user_create(&user_id)?;
            }
            UserCommands::List => {
                user_list()?;
            }
            UserCommands::Delete { user_id } => {
                user_delete(&user_id)?;
            }
            UserCommands::Show { user_id } => {
                user_show(&user_id)?;
            }
        },

        Commands::Workflow(command) => match command {
            WorkflowCommands::Load { file } => {
                commands::workflow::load(file)?;
            }

            WorkflowCommands::List => {
                commands::workflow::list()?;
            }

            WorkflowCommands::Show { workflow_id } => {
                commands::workflow::show(workflow_id)?;
            }

            WorkflowCommands::Delete { workflow_id } => {
                commands::workflow::delete(workflow_id)?;
            }
        },
    }
    Ok(())
}

fn user_create(user_id: &str) -> Result<()> {
    use crate::user_store::UserStore;

    if UserStore::user_exists(user_id)? {
        anyhow::bail!("User '{}' already exists", user_id);
    }

    UserStore::create_user(user_id)?;

    println!("\nUser '{}' can now sign records", user_id);
    println!("   Add roles with: ukweli user add-role {} <role>", user_id);

    Ok(())
}

fn user_list() -> Result<()> {
    use crate::user_store::UserStore;

    let users = UserStore::list_users()?;

    if users.is_empty() {
        println!("No users found.");
        println!("Create one with: ukweli user create <username>");
    } else {
        println!("Users:");
        for user in users {
            println!("  • {}", user);
        }
    }

    Ok(())
}

fn user_delete(user_id: &str) -> Result<()> {
    use crate::user_store::UserStore;

    println!("Are you sure you want to delete user '{}'?", user_id);
    println!("This will permanently delete their private key.");
    println!("Type 'yes (y)' to confirm:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" && input.trim() != "y" {
        println!("Cancelled.");
        return Ok(());
    }

    UserStore::delete_user(user_id)?;

    Ok(())
}

fn user_show(user_id: &str) -> Result<()> {
    use crate::user_store::UserStore;

    let user = UserStore::load_user(user_id)?;

    println!("User: {}", user.user_id);
    println!(
        "Verifying key: {}",
        hex::encode(user.verifying_key.to_bytes())
    );
    println!(
        "Roles: {}",
        if user.roles.is_empty() {
            "none".to_string()
        } else {
            user.roles
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    Ok(())
}
