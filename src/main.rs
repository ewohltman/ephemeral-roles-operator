pub use controller::*;

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

    tokio::spawn(async move {
        let store = reflector::store::Writer::<controller::Version>::default();
        let resource_client: Api<controller::Version> = Api::all(conn_client.clone());

        match reflector(store, watcher(resource_client, ListParams::default()))
            .try_for_each(|watch_event| async {
                handler::handle_resource(conn_client.clone(), watch_event).await;
                Ok(())
            })
            .await
        {
            Ok(_) => {}
            Err(err) => {
                println!("Reflector error: {:?}", err)
            }
        }
    })
    .await?;

    Ok(())
}
