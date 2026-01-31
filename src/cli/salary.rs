use crate::db::{models::*, repository::Repository};
use crate::money::Cents;
use chrono::{Local, NaiveDate};
use clap::Subcommand;
use inquire::{Confirm, CustomType, Select, Text};
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct SalaryDisplay {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Effective Date")]
    effective_date: String,
    #[tabled(rename = "Amount")]
    amount: String,
    #[tabled(rename = "Frequency")]
    frequency: String,
    #[tabled(rename = "Notes")]
    notes: String,
}

#[derive(Subcommand)]
pub(crate) enum SalaryCommands {
    /// Add a new salary entry
    Add {
        /// Effective date (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// Salary amount in major currency units (e.g., 50000 for £50,000)
        #[arg(short, long)]
        amount: Option<f64>,

        /// Currency code (e.g., USD, EUR, GBP)
        #[arg(short, long)]
        currency: Option<String>,

        /// Frequency (ANNUAL, MONTHLY, WEEKLY)
        #[arg(short, long)]
        frequency: Option<String>,

        /// Notes about the salary change
        #[arg(short, long)]
        notes: Option<String>,

        /// Skip confirmation prompt (automatically skip if all parameters provided)
        #[arg(long)]
        yes: bool,
    },
    /// Show current salary (most recent effective date)
    Current,
    /// List all salary entries
    List,
    /// Update a salary entry
    Update {
        /// ID of the salary entry to update
        id: i32,

        /// New effective date (YYYY-MM-DD)
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// New salary amount in major currency units
        #[arg(short, long)]
        amount: Option<f64>,

        /// New currency code
        #[arg(short, long)]
        currency: Option<String>,

        /// New frequency
        #[arg(short, long)]
        frequency: Option<String>,

        /// New notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// Delete a salary entry
    Delete {
        /// ID of the salary entry to delete
        id: Option<i32>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

pub(crate) async fn handle_command(
    cmd: &SalaryCommands,
    repo: Repository,
    config: crate::config::Config,
) -> color_eyre::Result<()> {
    match cmd {
        SalaryCommands::Add {
            date,
            amount,
            currency,
            frequency,
            notes,
            yes,
        } => {
            // Track if all params were provided via CLI
            let all_params_provided =
                date.is_some() && amount.is_some() && currency.is_some() && frequency.is_some();

            // Get effective date interactively if not provided
            let effective_date = match date {
                Some(d) => NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| {
                    color_eyre::eyre::eyre!("Invalid date format: {}. Use YYYY-MM-DD", e)
                })?,
                None => {
                    let today = Local::now().date_naive();
                    let default_date = today.format("%Y-%m-%d").to_string();

                    let date_str = Text::new(&format!(
                        "Effective date (YYYY-MM-DD, press Enter for today {}):",
                        default_date
                    ))
                    .with_default(&default_date)
                    .prompt()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                        color_eyre::eyre::eyre!("Invalid date format: {}. Use YYYY-MM-DD", e)
                    })?
                }
            };

            // Get amount interactively if not provided
            let amount = match amount {
                Some(a) => *a,
                None => CustomType::<f64>::new("Annual salary amount (e.g., 50000 for £50,000):")
                    .with_error_message("Please enter a valid number")
                    .prompt()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?,
            };

            // Get currency interactively if not provided
            let currency = match currency {
                Some(c) => c.clone(),
                None => {
                    let default_currency = config.application.default_currency.clone();
                    Text::new(&format!(
                        "Currency code (press Enter for {}):",
                        default_currency
                    ))
                    .with_default(&default_currency)
                    .prompt_skippable()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                    .unwrap_or(default_currency)
                }
            };

            // Get frequency interactively if not provided
            let frequency = match frequency {
                Some(f) => f.clone(),
                None => {
                    let frequencies = vec!["ANNUAL", "MONTHLY", "WEEKLY", "HOURLY"];
                    Select::new("Salary frequency:", frequencies)
                        .prompt()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?
                        .to_string()
                }
            };

            // Get notes interactively if not provided (optional field)
            let notes = match notes {
                Some(n) => Some(n.clone()),
                None => Text::new("Notes (optional - press Enter to skip):")
                    .prompt_skippable()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?,
            };

            // Show summary and ask for confirmation
            let currency_symbol = if currency == "GBP" { "£" } else { &currency };
            let formatted_amount = Cents::from_major_units(amount).format_currency(currency_symbol);

            println!("\nSalary entry to add:");
            println!("  Effective Date: {}", effective_date);
            println!("  Amount: {} {}", formatted_amount, frequency);
            println!("  Currency: {}", currency);
            if let Some(ref n) = notes {
                println!("  Notes: {}", n);
            }

            // Auto-skip confirmation if all params were provided via CLI or --yes flag
            let skip_confirmation = *yes || all_params_provided;

            if !skip_confirmation {
                let confirmed = Confirm::new("Add this salary entry?")
                    .with_default(true)
                    .prompt()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                if !confirmed {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let new_salary = NewSalary {
                effective_date,
                amount: Cents::from_major_units(amount),
                currency: currency.clone(),
                frequency,
                notes,
            };

            match repo.create_salary(new_salary).await {
                Ok(salary) => {
                    let currency_symbol = if salary.currency == "GBP" {
                        "£"
                    } else {
                        &salary.currency
                    };
                    println!(
                        "✓ Added salary entry effective {} with amount {} {}",
                        salary.effective_date,
                        salary.amount.format_currency(currency_symbol),
                        salary.frequency
                    );
                }
                Err(e) => {
                    eprintln!("Failed to add salary: {}", e);
                }
            }
        }
        SalaryCommands::Current => match repo.get_current_salary().await {
            Ok(Some(salary)) => {
                let currency_symbol = if salary.currency == "GBP" {
                    "£"
                } else {
                    &salary.currency
                };
                println!("Current salary (effective {}):", salary.effective_date);
                println!(
                    "  Amount: {} {}",
                    salary.amount.format_currency(currency_symbol),
                    salary.frequency
                );
                if let Some(notes) = &salary.notes {
                    println!("  Notes: {}", notes);
                }
            }
            Ok(None) => {
                println!("No salary entries found.");
            }
            Err(e) => {
                eprintln!("Failed to get current salary: {}", e);
            }
        },
        SalaryCommands::List => match repo.list_salaries().await {
            Ok(salaries) => {
                if salaries.is_empty() {
                    println!("No salary entries found.");
                } else {
                    let mut display_salaries = Vec::new();

                    for salary in salaries {
                        let currency_symbol = if salary.currency == "GBP" {
                            "£"
                        } else {
                            &salary.currency
                        };

                        display_salaries.push(SalaryDisplay {
                            id: salary.id,
                            effective_date: salary.effective_date.to_string(),
                            amount: salary.amount.format_currency(currency_symbol),
                            frequency: salary.frequency.clone(),
                            notes: salary.notes.unwrap_or_else(|| String::from("-")),
                        });
                    }

                    let table = Table::new(&display_salaries)
                        .with(Style::modern())
                        .to_string();
                    println!("{}", table);
                }
            }
            Err(e) => {
                eprintln!("Failed to list salaries: {}", e);
            }
        },
        SalaryCommands::Update {
            id,
            date,
            amount,
            currency,
            frequency,
            notes,
        } => {
            // Get existing salary
            match repo.get_salary_by_id(*id).await {
                Ok(Some(existing)) => {
                    // Get effective date (use existing if not provided)
                    let effective_date = match date {
                        Some(d) => NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| {
                            color_eyre::eyre::eyre!("Invalid date format: {}. Use YYYY-MM-DD", e)
                        })?,
                        None => existing.effective_date,
                    };

                    // Get amount (use existing if not provided)
                    let amount = match amount {
                        Some(a) => *a,
                        None => existing.amount.to_major_units(),
                    };

                    // Get currency (use existing if not provided)
                    let currency = currency.as_ref().unwrap_or(&existing.currency).clone();

                    // Get frequency (use existing if not provided)
                    let frequency = frequency.as_ref().unwrap_or(&existing.frequency).clone();

                    // Get notes (use provided value or existing)
                    let notes = notes.clone().or(existing.notes);

                    let updated_salary = NewSalary {
                        effective_date,
                        amount: Cents::from_major_units(amount),
                        currency: currency.clone(),
                        frequency,
                        notes,
                    };

                    match repo.update_salary(*id, updated_salary).await {
                        Ok(salary) => {
                            let currency_symbol = if salary.currency == "GBP" {
                                "£"
                            } else {
                                &salary.currency
                            };
                            println!("✓ Updated salary entry #{}", id);
                            println!("  Effective Date: {}", salary.effective_date);
                            println!(
                                "  Amount: {} {}",
                                salary.amount.format_currency(currency_symbol),
                                salary.frequency
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to update salary: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Salary entry with ID {} not found", id);
                }
                Err(e) => {
                    eprintln!("Failed to lookup salary: {}", e);
                }
            }
        }
        SalaryCommands::Delete { id, force } => {
            // Get salary ID interactively if not provided
            let id = match id {
                Some(i) => *i,
                None => {
                    match repo.list_salaries().await {
                        Ok(salaries) if !salaries.is_empty() => {
                            let salary_options: Vec<String> = salaries
                                .iter()
                                .map(|s| {
                                    let currency_symbol = if s.currency == "GBP" {
                                        "£"
                                    } else {
                                        &s.currency
                                    };
                                    let notes_preview = s
                                        .notes
                                        .as_ref()
                                        .map(|n| format!(" - {}", n))
                                        .unwrap_or_else(String::new);
                                    format!(
                                        "#{}: {} - {} {}{}",
                                        s.id,
                                        s.effective_date,
                                        s.amount.format_currency(currency_symbol),
                                        s.frequency,
                                        notes_preview
                                    )
                                })
                                .collect();

                            let selected =
                                Select::new("Select salary entry to delete:", salary_options)
                                    .prompt()
                                    .map_err(|e| {
                                        color_eyre::eyre::eyre!("Failed to get input: {}", e)
                                    })?;

                            // Extract ID from the selected string
                            selected
                                .split(':')
                                .next()
                                .and_then(|s| s.trim_start_matches('#').parse::<i32>().ok())
                                .ok_or_else(|| {
                                    color_eyre::eyre::eyre!("Failed to parse salary ID")
                                })?
                        }
                        Ok(_) => {
                            eprintln!("No salary entries found.");
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("Failed to list salaries: {}", e);
                            return Ok(());
                        }
                    }
                }
            };

            // Get the salary to display before deletion
            match repo.get_salary_by_id(id).await {
                Ok(Some(salary)) => {
                    if !force {
                        let currency_symbol = if salary.currency == "GBP" {
                            "£"
                        } else {
                            &salary.currency
                        };
                        let confirmed = Confirm::new(&format!(
                            "Are you sure you want to delete salary entry #{} (effective {}, {})?",
                            id,
                            salary.effective_date,
                            salary.amount.format_currency(currency_symbol)
                        ))
                        .with_default(false)
                        .prompt()
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to get input: {}", e))?;

                        if !confirmed {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }

                    match repo.delete_salary(id).await {
                        Ok(_) => println!("✓ Deleted salary entry #{}", id),
                        Err(e) => eprintln!("Failed to delete salary: {}", e),
                    }
                }
                Ok(None) => {
                    eprintln!("Salary entry with ID {} not found", id);
                }
                Err(e) => {
                    eprintln!("Failed to lookup salary: {}", e);
                }
            }
        }
    }

    Ok(())
}
