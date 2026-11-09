//! Shared SQL calls executed by native threads or a browser database worker.

mod ledger;

#[cfg(target_arch = "wasm32")]
mod browser;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

pub use ledger::{balance, debit, initialize, DebitError};

use async_sqlite::{rusqlite, Client, Error};
use std::sync::atomic::{AtomicBool, Ordering};

/// Exercises atomic writes, refusals and exact value transport on either backend.
pub async fn exercise(database: &Client) -> Result<(), String> {
    assert_eq!(
        balance(database).await.map_err(|error| error.to_string())?,
        100
    );
    assert_eq!(debit(database, 30).await.unwrap(), 70);
    for amount in [90, 0, -1] {
        assert!(matches!(
            debit(database, amount).await,
            Err(DebitError::InsufficientFunds)
        ));
    }
    assert_eq!(balance(database).await.unwrap(), 70);

    assert!(matches!(
        database
            .execute_batch(
                "BEGIN;
                 INSERT INTO audit VALUES (99);
                 INSERT INTO missing_table VALUES (1);
                 COMMIT;"
            )
            .await,
        Err(Error::Rusqlite(_))
    ));
    let audit = database
        .query_map("SELECT amount FROM audit", [], |row| row.get::<i64>(0))
        .await
        .unwrap();
    assert_eq!(audit, vec![30]);

    let integers = [
        i64::MIN,
        -9_007_199_254_740_993,
        9_007_199_254_740_993,
        i64::MAX,
    ];
    let bytes = vec![0, 255, 128, 1];
    database
        .execute_batch("CREATE TEMP TABLE samples (value INTEGER UNIQUE, payload BLOB)")
        .await
        .unwrap();
    for integer in integers {
        let changed = database
            .execute(
                "INSERT INTO samples VALUES (?1, ?2)",
                rusqlite::params![integer, &bytes],
            )
            .await
            .unwrap();
        assert_eq!(changed, 1);
    }
    assert!(matches!(
        database.execute("INSERT INTO samples VALUES (?1, ?2)", rusqlite::params![integers[0], &bytes]).await,
        Err(Error::Rusqlite(rusqlite::Error::SqliteFailure(error, _)))
            if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    ));
    assert!(matches!(
        database
            .query_row("SELECT ?", [], |row| row.get::<i64>(0))
            .await,
        Err(Error::Rusqlite(rusqlite::Error::InvalidParameterCount(
            0, 1
        )))
    ));
    let returned = database
        .query_map(
            "SELECT value, payload FROM samples ORDER BY rowid",
            [],
            |row| Ok((row.get::<i64>(0)?, row.get::<Vec<u8>>(1)?)),
        )
        .await
        .unwrap();
    assert_eq!(returned, integers.map(|integer| (integer, bytes.clone())));

    let mapped = AtomicBool::new(false);
    let value = database
        .query_row("SELECT ?1 AS value", [42], |row| {
            mapped.store(true, Ordering::Relaxed);
            row.get_named::<i64>("value")
        })
        .await
        .unwrap();
    assert_eq!((value, mapped.load(Ordering::Relaxed)), (42, true));

    let replies = futures_util::future::join_all(
        (0..12).map(|value| database.query_row("SELECT ?1", [value], |row| row.get::<i64>(0))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert_eq!(replies, (0..12).collect::<Vec<_>>());
    Ok(())
}
