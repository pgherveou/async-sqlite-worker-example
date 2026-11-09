use super::*;
use async_sqlite::ClientBuilder;

#[test]
fn shared_ledger_contract_on_native() {
    smol::block_on(async {
        let client = ClientBuilder::new()
            .open_with(|connection| initialize(connection).map_err(Into::into))
            .await
            .unwrap();
        exercise(&client).await.unwrap();
        client.close().await.unwrap();
    });
}
