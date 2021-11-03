use crate::deployer;
use kube::{
    api::{Resource, ResourceExt},
    runtime::watcher::Event,
    Client,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{fmt::Debug, hash::Hash};

pub async fn handle_resource<T>(conn_client: Client, watch_event: Event<T>)
where
    T: Resource + ResourceExt + Serialize + DeserializeOwned + Clone + Debug + Send + 'static,
    T::DynamicType: Eq + Hash + Clone + Default,
{
    match watch_event {
        Event::Applied(object) => handle_created(conn_client, object).await,
        Event::Deleted(object) => handle_deleted(conn_client, object).await,
        Event::Restarted(_) => println!("Resource watcher started"),
    }
}

pub async fn handle_created<T>(conn_client: Client, version: T)
where
    T: Resource + ResourceExt + Serialize + DeserializeOwned + Clone + Debug + Send + 'static,
{
    println!("Version applied: {}", version.name());

    match deployer::create_object(conn_client).await {
        Ok(object) => println!(
            "Object created: {}/{}",
            object.namespace().unwrap_or_else(|| "unknown".to_string()),
            object.name()
        ),
        Err(err) => println!("Error creating object: {:?}", err),
    }
}

pub async fn handle_deleted<T>(conn_client: Client, version: T)
where
    T: Resource + ResourceExt + Serialize + DeserializeOwned + Clone + Debug + Send + 'static,
{
    println!("Version deleted: {}", version.name());

    match deployer::delete_object(conn_client).await {
        Ok(object) => println!(
            "Object deleted: {}/{}",
            object.namespace().unwrap_or_else(|| "unknown".to_string()),
            object.name(),
        ),
        Err(err) => {
            println!("Error deleting object: {:?}", err);
        }
    }
}
