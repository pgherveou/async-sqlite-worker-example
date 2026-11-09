use super::*;
use async_sqlite::ClientBuilder;

#[test]
fn shared_ledger_contract_on_native() {
    smol::block_on(async {
        let client = ClientBuilder::new().open().await.unwrap();
        client.conn(initialize).await.unwrap();
        exercise(&client).await.unwrap();
        client.close().await.unwrap();
    });
}
