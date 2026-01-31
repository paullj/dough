mod account;
mod log;
mod salary;
mod update;

use account::AccountCommands;
use clap::{Parser, Subcommand};
use salary::SalaryCommands;

#[derive(Parser)]
#[command(
    about = "🍩 dough. personal finance tracker",
    version = clap::crate_version!(),
    author = clap::crate_authors!()
)]
struct Cli {
    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Log a new entry for each account
    Log {
        /// Date for the log entry (defaults to today, format: YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,

        /// Non-interactive mode: provide entries as account=amount pairs
        #[arg(value_parser = parse_entry)]
        entries: Vec<(String, f64)>,
    },
    /// Manage your accounts
    Account {
        #[command(subcommand)]
        subcommand: AccountCommands,
    },
    /// Manage salary information
    Salary {
        #[command(subcommand)]
        subcommand: SalaryCommands,
    },
    /// Update dough to the latest version
    Update,
}

fn parse_entry(s: &str) -> Result<(String, f64), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid entry format. Use: account=amount".to_string());
    }

    let account = parts[0].to_string();
    let amount = parts[1]
        .parse::<f64>()
        .map_err(|_| format!("Invalid amount: {}", parts[1]))?;

    Ok((account, amount))
}

pub(crate) async fn run() -> color_eyre::Result<()> {
    let args = Cli::parse();

    // Check for updates on startup (non-blocking)
    tokio::spawn(async {
        if let Ok(Some(new_version)) = update::check_update().await {
            eprintln!(
                "Update available: v{} (run 'dough update' to upgrade)",
                new_version
            );
        }
    });

    // Load configuration
    let config = crate::config::Config::load(args.config.as_deref())?;

    // Initialize database from config
    let pool = crate::db::init_pool(&config.data.path).await?;
    let repo = crate::db::repository::Repository::new(pool);

    match &args.command {
        None => {
            todo!("implement tui");
        }
        Some(Commands::Account { subcommand }) => {
            account::handle_command(subcommand, repo, config).await?;
        }
        Some(Commands::Salary { subcommand }) => {
            salary::handle_command(subcommand, repo, config).await?;
        }
        Some(Commands::Log { date, entries }) => {
            log::handle_command(date.as_deref(), entries, repo, config).await?;
        }
        Some(Commands::Update) => {
            update::handle_command().await?;
        }
    }

    Ok(())
}
