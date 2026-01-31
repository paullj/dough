use crate::money::Cents;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub public_id: String,
    pub name: String,
    pub account_type: String,
    pub provider: Option<String>,
    pub currency: String,
    #[sqlx(rename = "initial_balance_cents")]
    pub initial_balance: Cents,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub name: String,
    pub account_type: String,
    pub provider: Option<String>,
    pub currency: String,
    pub initial_balance: Cents,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Salary {
    pub id: i32,
    pub effective_date: NaiveDate,
    #[sqlx(rename = "amount_cents")]
    pub amount: Cents,
    pub currency: String,
    pub frequency: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSalary {
    pub effective_date: NaiveDate,
    pub amount: Cents,
    pub currency: String,
    pub frequency: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountEntry {
    pub id: i32,
    pub account_id: i32,
    #[sqlx(rename = "amount_cents")]
    pub amount: Cents,
    pub entry_date: NaiveDate,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccountEntry {
    pub account_id: i32,
    pub amount: Cents,
    pub entry_date: NaiveDate,
    pub description: Option<String>,
}
