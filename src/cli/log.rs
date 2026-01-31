use crate::config::Config;
use crate::db::models::{Account, NewAccountEntry};
use crate::db::repository::Repository;
use crate::money::Cents;
use chrono::{Local, NaiveDate};
use color_eyre::eyre::{Result, eyre};
use colored::Colorize;
use inquire::{Confirm, Text};
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct LogSummary {
    #[tabled(rename = "Account")]
    account: String,
    #[tabled(rename = "Previous Balance")]
    previous_balance: String,
    #[tabled(rename = "Current Balance")]
    current_balance: String,
    #[tabled(rename = "Change")]
    change: String,
    #[tabled(rename = "Change %")]
    change_percent: String,
}

pub async fn handle_command(
    date_str: Option<&str>,
    entries: &[(String, f64)],
    repo: Repository,
    config: Config,
) -> Result<()> {
    // Parse date or use today
    let entry_date = if let Some(date_str) = date_str {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| eyre!("Invalid date format. Use YYYY-MM-DD"))?
    } else {
        Local::now().naive_local().date()
    };

    // Check if date is in the future
    if entry_date > Local::now().naive_local().date() {
        return Err(eyre!("Cannot log entries for future dates"));
    }

    // Get all active accounts
    let accounts = repo.list_accounts(true).await?;

    if accounts.is_empty() {
        return Err(eyre!(
            "No active accounts found. Please add accounts first using 'dough account add'"
        ));
    }

    // Filter accounts that were active on the given date
    let active_accounts: Vec<Account> = accounts
        .into_iter()
        .filter(|a| a.created_at.date_naive() <= entry_date)
        .collect();

    if active_accounts.is_empty() {
        return Err(eyre!("No accounts were active on {}", entry_date));
    }

    let mut log_entries = Vec::new();

    if entries.is_empty() {
        // Interactive mode
        println!("Logging entries for {}", entry_date.format("%Y-%m-%d"));
        println!("Enter the current balance for each account:");
        println!();

        // Group accounts by provider
        let mut grouped: std::collections::BTreeMap<String, Vec<&Account>> =
            std::collections::BTreeMap::new();
        for account in &active_accounts {
            let provider = account
                .provider
                .clone()
                .unwrap_or_else(|| "No Provider".to_string());
            grouped
                .entry(provider)
                .or_insert_with(Vec::new)
                .push(account);
        }

        // Prompt for each group
        for (provider, accts) in grouped {
            if provider != "No Provider" {
                println!("\n{}", format!("=== {} ===", provider).blue());
            }

            for account in accts {
                // Get current balance for reference
                let current_balance = repo
                    .get_account_balance(account.id, Some(entry_date))
                    .await?;

                let prompt = format!(
                    "{} ({}): current calculated balance = {}",
                    account.name,
                    account.account_type,
                    current_balance.format_currency(&account.currency)
                );

                let input = Text::new(&prompt)
                    .with_help_message("Enter total balance (leave blank to skip)")
                    .prompt_skippable()?;

                if let Some(amount_str) = input {
                    let total_balance = amount_str
                        .parse::<f64>()
                        .map_err(|_| eyre!("Invalid amount: {}", amount_str))?;

                    // Calculate the difference needed to reach this total
                    let total_cents = Cents::from_major_units(total_balance);
                    let difference = total_cents - current_balance;

                    if difference.0 != 0 {
                        log_entries.push((
                            account.clone(),
                            current_balance,
                            difference,
                            total_cents,
                        ));
                    }
                }
            }
        }
    } else {
        // Non-interactive mode with provided entries
        for (account_name, amount) in entries {
            let account = active_accounts
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(account_name))
                .ok_or_else(|| {
                    eyre!(
                        "Account '{}' not found or not active on {}",
                        account_name,
                        entry_date
                    )
                })?
                .clone();

            let current_balance = repo
                .get_account_balance(account.id, Some(entry_date))
                .await?;
            let total_balance = Cents::from_major_units(*amount);
            let difference = total_balance - current_balance;

            if difference.0 != 0 {
                log_entries.push((account, current_balance, difference, total_balance));
            }
        }
    }

    if log_entries.is_empty() {
        println!("No entries to log.");
        return Ok(());
    }

    // Show summary
    println!("\nEntry Summary for {}:", entry_date.format("%Y-%m-%d"));

    let summary: Vec<LogSummary> = log_entries
        .iter()
        .map(|(account, previous_balance, difference, new_balance)| {
            // Calculate percentage change
            let percent_change = if previous_balance.0 != 0 {
                (difference.0 as f64 / previous_balance.0 as f64) * 100.0
            } else if difference.0 != 0 {
                100.0
            } else {
                0.0
            };

            // Format percentage with color based on threshold
            let percent_str = format!("{:+.1}%", percent_change);
            let colored_percent = if percent_change.abs() >= config.ui.percentage_threshold {
                if percent_change > 0.0 {
                    percent_str.green().to_string()
                } else {
                    percent_str.red().to_string()
                }
            } else {
                percent_str
            };

            // Format change amount with color
            let change_str = difference.format_currency(&account.currency);
            let colored_change = if difference.0 > 0 {
                format!("+{}", change_str).green().to_string()
            } else if difference.0 < 0 {
                change_str.red().to_string()
            } else {
                change_str
            };

            LogSummary {
                account: account.name.clone(),
                previous_balance: previous_balance.format_currency(&account.currency),
                current_balance: new_balance.format_currency(&account.currency),
                change: colored_change,
                change_percent: colored_percent,
            }
        })
        .collect();

    let table = Table::new(&summary).with(Style::modern()).to_string();
    println!("{}", table);

    // Confirm before saving
    let confirm = if entries.is_empty() {
        Confirm::new("Save these entries?")
            .with_default(true)
            .prompt()?
    } else {
        true // Non-interactive mode, auto-confirm
    };

    if confirm {
        // Save entries
        for (account, _previous, amount, _new) in log_entries {
            let entry = NewAccountEntry {
                account_id: account.id,
                amount,
                entry_date,
                description: Some(format!("Daily balance log")),
            };

            repo.create_account_entry(entry).await?;
        }

        println!("Entries saved successfully!");
    } else {
        println!("Entries cancelled.");
    }

    Ok(())
}
