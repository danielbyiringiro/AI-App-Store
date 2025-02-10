use clap::{Parser, Subcommand};
use crate::db::{PermissionDB, PermissionEntry, PermissionState};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Request {
        #[arg(short, long)]
        app: String,
        #[arg(
            short, 
            long,
            required = true,
            num_args = 1..,
            value_delimiter = ','
        )]
        permissions: Vec<String>,
    },
}

pub fn run_cli() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut db = PermissionDB::init()?;

    match args.command {
        Command::Request { app, permissions } => {
            let entry = PermissionEntry {
                app_name: app,
                requested_permissions: permissions,
                permission_state: PermissionState::Block,
            };
            db.upsert(&entry)?;
            println!("Permission request submitted");
        }
    }

    Ok(())
}
