use async_sqlite::{rusqlite, Client, Error};

/// A debit was refused or its database operation failed.
#[derive(Debug)]
pub enum DebitError {
    /// The debit exceeds the available balance or is not positive.
    InsufficientFunds,
    /// The database could not confirm the debit.
    Database(Error),
}

impl core::fmt::Display for DebitError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientFunds => formatter.write_str("insufficient funds"),
            Self::Database(error) => error.fmt(formatter),
        }
    }
}

/// Creates the example schema once, preserving data on subsequent opens.
pub fn initialize(connection: &rusqlite::Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS account (id INTEGER PRIMARY KEY, balance INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS audit (amount INTEGER NOT NULL);
         CREATE TRIGGER IF NOT EXISTS debit_audit AFTER UPDATE OF balance ON account
         WHEN NEW.balance < OLD.balance BEGIN
             INSERT INTO audit VALUES (OLD.balance - NEW.balance);
         END;
         INSERT OR IGNORE INTO account VALUES (1, 100);",
    )
}

/// Reads the persisted balance.
pub async fn balance(database: &Client) -> Result<i64, Error> {
    database
        .query_row("SELECT balance FROM account WHERE id = 1", [], |row| {
            row.get(0)
        })
        .await
}

/// Debits and audits atomically, refusing amounts outside the available balance.
pub async fn debit(database: &Client, amount: i64) -> Result<i64, DebitError> {
    database
        .query_row(
            "UPDATE account SET balance = balance - ?1
             WHERE id = 1 AND ?1 > 0 AND balance >= ?1 RETURNING balance",
            [amount],
            |row| row.get(0),
        )
        .await
        .map_err(|error| match error {
            Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows) => DebitError::InsufficientFunds,
            error => DebitError::Database(error),
        })
}
