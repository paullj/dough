use super::models::*;
use crate::id::generate_public_id;
use crate::money::Cents;
use chrono::NaiveDate;
use sqlx::{Row, SqlitePool};

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ========== Account Operations ==========

    pub async fn create_account(&self, account: NewAccount) -> Result<Account, sqlx::Error> {
        let initial_balance_cents = account.initial_balance.0;

        // Insert with NULL id to get autoincrement, then update with public_id
        let result = sqlx::query!(
            r#"
            INSERT INTO accounts (public_id, name, account_type, provider, currency, initial_balance_cents)
            VALUES ('temp', ?1, ?2, ?3, ?4, ?5)
            "#,
            account.name,
            account.account_type,
            account.provider,
            account.currency,
            initial_balance_cents
        )
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as i32;
        let public_id = generate_public_id(id);

        // Update with the generated public_id
        sqlx::query!(
            r#"
            UPDATE accounts SET public_id = ?1 WHERE id = ?2
            "#,
            public_id,
            id
        )
        .execute(&self.pool)
        .await?;

        self.get_account_by_id(id).await.map(|a| a.unwrap())
    }

    pub async fn get_account_by_id(&self, id: i32) -> Result<Option<Account>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", public_id, name, account_type, provider, currency, initial_balance_cents,
                   is_active, created_at, updated_at
            FROM accounts
            WHERE id = ?1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Account {
            id: r.id as i32,
            public_id: r.public_id,
            name: r.name,
            account_type: r.account_type,
            provider: r.provider,
            currency: r.currency,
            initial_balance: Cents(r.initial_balance_cents),
            is_active: r.is_active,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn get_account(&self, public_id: &str) -> Result<Option<Account>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", public_id, name, account_type, provider, currency, initial_balance_cents,
                   is_active, created_at, updated_at
            FROM accounts
            WHERE public_id = ?1
            "#,
            public_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Account {
            id: r.id as i32,
            public_id: r.public_id,
            name: r.name,
            account_type: r.account_type,
            provider: r.provider,
            currency: r.currency,
            initial_balance: Cents(r.initial_balance_cents),
            is_active: r.is_active,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn get_account_by_name(&self, name: &str) -> Result<Option<Account>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", public_id, name, account_type, provider, currency, initial_balance_cents,
                   is_active, created_at, updated_at
            FROM accounts
            WHERE name = ?1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Account {
            id: r.id as i32,
            public_id: r.public_id,
            name: r.name,
            account_type: r.account_type,
            provider: r.provider,
            currency: r.currency,
            initial_balance: Cents(r.initial_balance_cents),
            is_active: r.is_active,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn list_accounts(&self, active_only: bool) -> Result<Vec<Account>, sqlx::Error> {
        let query = if active_only {
            "SELECT id, public_id, name, account_type, provider, currency, initial_balance_cents,
                    is_active, created_at, updated_at
             FROM accounts
             WHERE is_active = TRUE
             ORDER BY provider, name"
        } else {
            "SELECT id, public_id, name, account_type, provider, currency, initial_balance_cents,
                    is_active, created_at, updated_at
             FROM accounts
             ORDER BY provider, name"
        };

        let rows = sqlx::query(query).fetch_all(&self.pool).await?;

        Ok(rows
            .iter()
            .map(|row| Account {
                id: row.get("id"),
                public_id: row.get("public_id"),
                name: row.get("name"),
                account_type: row.get("account_type"),
                provider: row.try_get("provider").ok(),
                currency: row.get("currency"),
                initial_balance: Cents(row.get("initial_balance_cents")),
                is_active: row.get("is_active"),
                created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc(),
                updated_at: row.get::<chrono::NaiveDateTime, _>("updated_at").and_utc(),
            })
            .collect())
    }

    #[allow(dead_code)]
    pub async fn update_account_status(
        &self,
        public_id: &str,
        is_active: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET is_active = ?1
            WHERE public_id = ?2
            "#,
            is_active,
            public_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_account(&self, public_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM accounts WHERE public_id = ?1
            "#,
            public_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========== Salary Operations ==========

    pub async fn create_salary(&self, salary: NewSalary) -> Result<Salary, sqlx::Error> {
        let amount_cents = salary.amount.0;

        let result = sqlx::query!(
            r#"
            INSERT INTO salaries (effective_date, amount_cents, currency, frequency, notes)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            salary.effective_date,
            amount_cents,
            salary.currency,
            salary.frequency,
            salary.notes
        )
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as i32;
        self.get_salary_by_id(id).await.map(|s| s.unwrap())
    }

    pub async fn get_salary_by_id(&self, id: i32) -> Result<Option<Salary>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", effective_date, amount_cents, currency, frequency, notes,
                   created_at, updated_at
            FROM salaries
            WHERE id = ?1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Salary {
            id: r.id as i32,
            effective_date: r.effective_date,
            amount: Cents(r.amount_cents),
            currency: r.currency,
            frequency: r.frequency,
            notes: r.notes,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn get_current_salary(&self) -> Result<Option<Salary>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", effective_date, amount_cents, currency, frequency, notes,
                   created_at, updated_at
            FROM salaries
            WHERE effective_date <= date('now')
            ORDER BY effective_date DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Salary {
            id: r.id as i32,
            effective_date: r.effective_date,
            amount: Cents(r.amount_cents),
            currency: r.currency,
            frequency: r.frequency,
            notes: r.notes,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn list_salaries(&self) -> Result<Vec<Salary>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id as "id!", effective_date, amount_cents, currency, frequency, notes,
                   created_at, updated_at
            FROM salaries
            ORDER BY effective_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Salary {
                id: r.id as i32,
                effective_date: r.effective_date,
                amount: Cents(r.amount_cents),
                currency: r.currency,
                frequency: r.frequency,
                notes: r.notes,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
            })
            .collect())
    }

    pub async fn update_salary(&self, id: i32, salary: NewSalary) -> Result<Salary, sqlx::Error> {
        let amount_cents = salary.amount.0;

        sqlx::query!(
            r#"
            UPDATE salaries
            SET effective_date = ?1, amount_cents = ?2, currency = ?3, frequency = ?4, notes = ?5
            WHERE id = ?6
            "#,
            salary.effective_date,
            amount_cents,
            salary.currency,
            salary.frequency,
            salary.notes,
            id
        )
        .execute(&self.pool)
        .await?;

        self.get_salary_by_id(id).await.map(|s| s.unwrap())
    }

    pub async fn delete_salary(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM salaries WHERE id = ?1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========== Account Entry Operations ==========

    pub async fn create_account_entry(
        &self,
        entry: NewAccountEntry,
    ) -> Result<AccountEntry, sqlx::Error> {
        let amount_cents = entry.amount.0;

        let result = sqlx::query!(
            r#"
            INSERT INTO account_entries (account_id, amount_cents, entry_date, description)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            entry.account_id,
            amount_cents,
            entry.entry_date,
            entry.description
        )
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid() as i32;
        self.get_account_entry_by_id(id).await.map(|e| e.unwrap())
    }

    pub async fn get_account_entry_by_id(
        &self,
        id: i32,
    ) -> Result<Option<AccountEntry>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id as "id!", account_id, amount_cents, entry_date, description,
                   created_at, updated_at
            FROM account_entries
            WHERE id = ?1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| AccountEntry {
            id: r.id as i32,
            account_id: r.account_id as i32,
            amount: Cents(r.amount_cents),
            entry_date: r.entry_date,
            description: r.description,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        }))
    }

    pub async fn list_account_entries(
        &self,
        account_id: i32,
        date: Option<NaiveDate>,
    ) -> Result<Vec<AccountEntry>, sqlx::Error> {
        match date {
            Some(date) => {
                let rows = sqlx::query!(
                    r#"
                    SELECT id as "id!", account_id, amount_cents, entry_date, description,
                           created_at, updated_at
                    FROM account_entries
                    WHERE account_id = ?1 AND entry_date = ?2
                    ORDER BY created_at DESC
                    "#,
                    account_id,
                    date
                )
                .fetch_all(&self.pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|r| AccountEntry {
                        id: r.id as i32,
                        account_id: r.account_id as i32,
                        amount: Cents(r.amount_cents),
                        entry_date: r.entry_date,
                        description: r.description,
                        created_at: r.created_at.and_utc(),
                        updated_at: r.updated_at.and_utc(),
                    })
                    .collect())
            }
            None => {
                let rows = sqlx::query!(
                    r#"
                    SELECT id as "id!", account_id, amount_cents, entry_date, description,
                           created_at, updated_at
                    FROM account_entries
                    WHERE account_id = ?1
                    ORDER BY entry_date DESC, created_at DESC
                    "#,
                    account_id
                )
                .fetch_all(&self.pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|r| AccountEntry {
                        id: r.id as i32,
                        account_id: r.account_id as i32,
                        amount: Cents(r.amount_cents),
                        entry_date: r.entry_date,
                        description: r.description,
                        created_at: r.created_at.and_utc(),
                        updated_at: r.updated_at.and_utc(),
                    })
                    .collect())
            }
        }
    }

    pub async fn get_account_balance(
        &self,
        account_id: i32,
        date: Option<NaiveDate>,
    ) -> Result<Cents, sqlx::Error> {
        // Get initial balance
        let initial = sqlx::query!(
            r#"
            SELECT initial_balance_cents
            FROM accounts
            WHERE id = ?1
            "#,
            account_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Sum all entries up to the given date (or all if no date)
        let sum_total = match date {
            Some(date) => {
                let sum = sqlx::query!(
                    r#"
                    SELECT COALESCE(SUM(amount_cents), 0) as total
                    FROM account_entries
                    WHERE account_id = ?1 AND entry_date <= ?2
                    "#,
                    account_id,
                    date
                )
                .fetch_one(&self.pool)
                .await?;
                sum.total
            }
            None => {
                let sum = sqlx::query!(
                    r#"
                    SELECT COALESCE(SUM(amount_cents), 0) as total
                    FROM account_entries
                    WHERE account_id = ?1
                    "#,
                    account_id
                )
                .fetch_one(&self.pool)
                .await?;
                sum.total
            }
        };

        Ok(Cents(initial.initial_balance_cents + sum_total))
    }
}
