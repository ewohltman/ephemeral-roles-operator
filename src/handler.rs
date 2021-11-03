use crate::{deployer, ephemeral_roles};
use kube::{api::ResourceExt, runtime::watcher::Event, Client};

pub async fn handle(conn_client: Client, watch_event: Event<ephemeral_roles::ERVersion>) {
    match watch_event {
        Event::Applied(er_version) => handle_created(conn_client, er_version).await,
        Event::Deleted(er_version) => handle_deleted(conn_client, er_version).await,
        Event::Restarted(_) => println!("Resource watcher started"),
    }
}

pub async fn handle_created(conn_client: Client, er_version: ephemeral_roles::ERVersion) {
    println!("ERVersion applied: {}", er_version.name());

    match deployer::deploy(conn_client, er_version.name().as_str()).await {
        Ok(_) => println!("ERVersion {} deployed successfully", er_version.name()),
        Err(err) => println!("Error deploying ERVersion {}: {:?}", er_version.name(), err),
    }
}

pub async fn handle_deleted(conn_client: Client, er_version: ephemeral_roles::ERVersion) {
    println!("ERVersion deleted: {}", er_version.name());

    match deployer::remove(conn_client).await {
        Ok(_) => println!("ERVersion {} removed successfully", er_version.name()),
        Err(err) => println!("Error removing ERVersion {}: {:?}", er_version.name(), err),
    }
}
