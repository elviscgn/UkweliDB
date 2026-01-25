mod commands;
mod config;
mod ledger_manager;
mod user_store;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// TODO
// init – initialise database and genesis record #10
// record append – append signed records to the ledger #11
// record verify – verify hash chain and signatures #12
// workflow load – load JSON/YAML workflow definitions #13
// workflow list – list available workflows #14
// record show – inspect records by entity or range #15
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
    // /// user management comms
    // User {
    //     #[command(subcommand)]
    //     command: UserCommands,
    // },
    #[command(subcommand)]
    Record(RecordCommands),
    // #[command(subcommand)]
    // Workflow(WorkflowCommands),

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

    List,
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
                println!("TODO: Verify chain");
            }
            RecordCommands::Show { index } => {
                println!("TODO: Show record {}", index);
            }
            RecordCommands::List => {
                println!("TODO: List all records");
            }
        },
    }
    Ok(())
}
