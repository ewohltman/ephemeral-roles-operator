use futures::prelude::*;
use kube::{
    api::{Api, ListParams},
    runtime::watcher,
    Client,
};
use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Started ephemeral-roles-operator");

    let conn_client = Client::try_default().await?;
    let resource_client: Api<operator::ERVersion> = Api::all(conn_client.clone());

    let list_params = ListParams::default();
    let watcher = watcher(resource_client, list_params).boxed();

    let join_handle = tokio::spawn(async move {
        let run_watcher = watcher.try_for_each(|event| async {
            let run_handler = operator::handle(conn_client.clone(), event);
            if let Err(err) = run_handler.await {
                println!("Handler error: {:?}", err)
            };

            Ok(())
        });
        if let Err(err) = run_watcher.await {
            println!("Watcher error: {:?}", err)
        }
    });

    join_handle.await?;

    Ok(())
}
