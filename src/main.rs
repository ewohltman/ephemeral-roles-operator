use futures::prelude::*;
use kube::{
    api::{Api, ListParams},
    runtime::{reflector, watcher},
    Client,
};
use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Started ephemeral-roles-operator");

    let conn_client = Client::try_default().await?;
    let join_handle = tokio::spawn(async move {
        let store = reflector::store::Writer::<operator::ERVersion>::default();
        let resource_client: Api<operator::ERVersion> = Api::all(conn_client.clone());
        let reflector = reflector(store, watcher(resource_client, ListParams::default()))
            .try_for_each(|event| async {
                operator::handle(conn_client.clone(), event).await;
                Ok(())
            });

        match reflector.await {
            Ok(_) => {}
            Err(err) => {
                println!("Reflector error: {:?}", err)
            }
        }
    });

    join_handle.await?;

    Ok(())
}
