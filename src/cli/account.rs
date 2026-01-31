use clap::Subcommand;
use inquire::{Text, Select, CustomType, Confirm};
use crate::db::{models::*, repository::Repository};
use crate::money::Cents;
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct AccountDisplay {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Current Balance")]
    current_balance: String,
}

#[derive(Subcommand)]
pub(crate) enum AccountCommands {
    /// Add a new account to track
    Add {
        /// Account name
        #[arg(short, long)]
        name: Option<String>,

        /// Account type/category (e.g., checking, savings, credit card, etc.)
        #[arg(short = 't', long)]
        account_type: Option<String>,

        /// Provider/institution (e.g., Chase, Vanguard, Lloyds)
        #[arg(short = 'p', long)]
        provider: Option<String>,

        /// Initial balance in major currency units (e.g., 100.50 for £100.50)
        #[arg(short = 'b', long)]
        initial_balance: Option<f64>,

        /// Currency code (e.g., USD, EUR)
        #[arg(short, long)]
        currency: Option<String>,
    },
    /// List existing accounts
    List {
        /// Show only active accounts
        #[arg(short, long)]
        active: bool,
    },
    /// Remove an account
    Remove {
        /// Account name to remove
        #[arg(short, long)]
        name: Option<String>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Delete an account by its public ID
    Delete {
        /// Public ID of the account to delete
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

pub(crate) async fn handle_command(
    cmd: &AccountCommands,
    repo: Repository,
    config: crate::config::Config,
) -> color_eyre::Result<()> {
    match cmd {
        AccountCommands::Add {
            name,
            account_type,
            provider,
            initial_balance,
            currency,
        } => {
            // Get account name interactively if not provided
            let name = match name {
                Some(n) => n.clone(),
                None => Text::new("Account name:")
                    .prompt()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?,
            };

            // Get account type interactively if not provided
            let account_type = match account_type {
                Some(t) => t.clone(),
                None => {
                    Text::new("Account type/category (e.g., checking, savings, credit card):")
                        .prompt()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                }
            };

            // Validate that account type is not empty
            if account_type.trim().is_empty() {
                eprintln!("Account type cannot be empty");
                return Ok(());
            }

            // Get provider interactively if not provided (optional field)
            let provider = match provider {
                Some(p) => Some(p.clone()),
                None => {
                    Text::new("Provider/Institution (e.g., Chase, Vanguard) - press Enter to skip:")
                        .prompt_skippable()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                        .filter(|s| !s.trim().is_empty())
                }
            };

            // Get initial balance interactively if not provided (optional field)
            let initial_balance = match initial_balance {
                Some(b) => *b,
                None => {
                    CustomType::<f64>::new("Initial balance (press Enter for 0):")
                        .with_default(0.0)
                        .with_error_message("Please enter a valid number")
                        .prompt_skippable()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                        .unwrap_or(0.0)
                }
            };

            // Get currency interactively if not provided (optional field)
            let currency = match currency {
                Some(c) => c.clone(),
                None => {
                    let default_currency = config.application.default_currency.clone();
                    Text::new(&format!("Currency code (press Enter for {}):", default_currency))
                        .with_default(&default_currency)
                        .prompt_skippable()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                        .unwrap_or(default_currency)
                }
            };

            let new_account = NewAccount {
                name: name.clone(),
                account_type,
                provider,
                currency: currency.clone(),
                initial_balance: Cents::from_major_units(initial_balance),
            };

            match repo.create_account(new_account).await {
                Ok(account) => {
                    let currency_symbol = if account.currency == "GBP" { "£" } else { &account.currency };
                    println!("✓ Added account '{}' with initial balance {}",
                        account.name,
                        account.initial_balance.format_currency(currency_symbol));
                }
                Err(e) => {
                    eprintln!("Failed to add account: {}", e);
                }
            }
        }
        AccountCommands::List { active } => {
            match repo.list_accounts(*active).await {
                Ok(accounts) => {
                    if accounts.is_empty() {
                        if *active {
                            println!("No active accounts found.");
                        } else {
                            println!("No accounts found.");
                        }
                    } else {
                        // Group accounts by provider
                        let mut grouped: std::collections::BTreeMap<String, Vec<_>> = std::collections::BTreeMap::new();
                        for account in accounts {
                            let provider = account.provider.clone()
                                .filter(|p| !p.trim().is_empty())
                                .unwrap_or_else(|| "No Provider".to_string());
                            grouped.entry(provider).or_insert_with(Vec::new).push(account);
                        }

                        // Display grouped accounts
                        for (provider, accts) in grouped {
                            println!("\n{}", if provider == "No Provider" {
                                "No Provider".to_string()
                            } else {
                                format!("Provider: {}", provider)
                            });
                            println!("{}", "-".repeat(60));

                            let mut display_accounts = Vec::new();
                            for account in accts {
                                let currency_symbol = if account.currency == "GBP" { "£" } else { &account.currency };

                                // Get actual balance including entries
                                let balance = repo.get_account_balance(account.id, None).await
                                    .unwrap_or(account.initial_balance);

                                display_accounts.push(AccountDisplay {
                                    id: account.public_id.clone(),
                                    name: account.name.clone(),
                                    current_balance: balance.format_currency(currency_symbol),
                                });
                            }

                            let table = Table::new(&display_accounts)
                                .with(Style::modern())
                                .to_string();
                            println!("{}", table);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list accounts: {}", e);
                }
            }
        }
        AccountCommands::Remove { name, force } => {
            // Get account name interactively if not provided
            let name = match name {
                Some(n) => n.clone(),
                None => {
                    // First, list available accounts for user to choose from
                    match repo.list_accounts(false).await {
                        Ok(accounts) if !accounts.is_empty() => {
                            let account_names: Vec<String> = accounts.iter()
                                .map(|a| format!("{} ({} - {})",
                                    a.name,
                                    a.account_type,
                                    a.initial_balance.format_currency(
                                        if a.currency == "GBP" { "£" } else { &a.currency }
                                    )
                                ))
                                .collect();

                            let selected = Select::new("Select account to remove:", account_names)
                                .prompt()
                                .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                            // Extract just the account name from the selected string
                            accounts.iter()
                                .find(|a| selected.starts_with(&a.name))
                                .map(|a| a.name.clone())
                                .unwrap_or_else(|| selected.split(" (").next().unwrap_or("").to_string())
                        }
                        Ok(_) => {
                            eprintln!("No accounts found.");
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("Failed to list accounts: {}", e);
                            return Ok(());
                        }
                    }
                }
            };

            match repo.get_account_by_name(&name).await {
                Ok(Some(account)) => {
                    if !force {
                        let confirmed = Confirm::new(&format!("Are you sure you want to delete account '{}'?", name))
                            .with_default(false)
                            .prompt()
                            .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                        if !confirmed {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }

                    match repo.delete_account(&account.public_id).await {
                        Ok(_) => println!("✓ Removed account '{}'", name),
                        Err(e) => eprintln!("Failed to remove account: {}", e),
                    }
                }
                Ok(None) => {
                    eprintln!("Account '{}' not found", name);
                }
                Err(e) => {
                    eprintln!("Failed to lookup account: {}", e);
                }
            }
        }
        AccountCommands::Delete { id, force } => {
            // Try to get the account by public_id
            match repo.get_account(id).await {
                Ok(Some(account)) => {
                    if !force {
                        let confirmed = Confirm::new(&format!("Are you sure you want to delete account '{}' (ID: {})?", account.name, id))
                            .with_default(false)
                            .prompt()
                            .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                        if !confirmed {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }

                    match repo.delete_account(id).await {
                        Ok(_) => println!("✓ Deleted account '{}' (ID: {})", account.name, id),
                        Err(e) => eprintln!("Failed to delete account: {}", e),
                    }
                }
                Ok(None) => {
                    eprintln!("Account with ID '{}' not found", id);
                }
                Err(e) => {
                    eprintln!("Failed to lookup account: {}", e);
                }
            }
        }
    }

    Ok(())
}

