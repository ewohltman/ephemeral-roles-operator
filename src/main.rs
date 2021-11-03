pub use controller::*;

use futures::prelude::*;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::core::params::{DeleteParams, PostParams};
use kube::core::ObjectMeta;
use kube::{
    api::{Api, ListParams, Resource, ResourceExt},
    runtime::{reflector, watcher, watcher::Event},
    Client,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{error, fmt::Debug, hash::Hash};

const TEST_OBJECT: &str = "test-object";
const NAMESPACE: &str = "ephemeral-roles";

async fn manage<T, K>(
    conn_client: Client,
    resource_client: Api<T>,
    resource_handler: fn(Client, Event<T>) -> K,
) where
    T: Resource + Clone + DeserializeOwned + Debug + Send + 'static,
    T::DynamicType: Eq + Hash + Clone + Default,
    K: Future + Send + 'static,
    K::Output: Send + 'static,
{
    let store = reflector::store::Writer::<T>::default();

    match reflector(store, watcher(resource_client, ListParams::default()))
        .try_for_each(|watch_event| async {
            resource_handler(conn_client.clone(), watch_event).await;
            Ok(())
        })
        .await
    {
        Ok(_) => {}
        Err(err) => {
            println!("Reflector error: {:?}", err)
        }
    }
}

async fn resource_handler<T>(conn_client: Client, watch_event: Event<T>)
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

async fn handle_created<T>(conn_client: Client, object: T)
where
    T: Resource + ResourceExt + Serialize + DeserializeOwned + Clone + Debug + Send + 'static,
{
    println!("Version applied: {}", object.name());

    match create_object(conn_client).await {
        Ok(object) => println!(
            "Object created: {}/{}",
            object.namespace().unwrap_or("unknown".to_string()),
            object.name()
        ),
        Err(err) => println!("Error creating object: {:?}", err),
    }
}

async fn handle_deleted<T>(conn_client: Client, object: T)
where
    T: Resource + ResourceExt + Serialize + DeserializeOwned + Clone + Debug + Send + 'static,
{
    println!("Version deleted: {}", object.name());

    match delete_object(conn_client).await {
        Ok(object) => println!(
            "Object deleted: {}/{}",
            object.namespace().unwrap_or("unknown".to_string()),
            object.name()
        ),
        Err(err) => {
            println!("Error deleting object: {:?}", err);
            return;
        }
    }
}

async fn create_object(conn_client: Client) -> Result<ConfigMap, kube::Error> {
    let client: Api<ConfigMap> = Api::namespaced(conn_client, NAMESPACE);

    let mut metadata: ObjectMeta = Default::default();
    metadata.name = Some(TEST_OBJECT.to_string());

    client
        .create(
            &PostParams::default(),
            &ConfigMap {
                metadata,
                data: None,
                binary_data: None,
                immutable: None,
            },
        )
        .await
}

async fn delete_object(conn_client: Client) -> Result<ConfigMap, kube::Error> {
    let client: Api<ConfigMap> = Api::namespaced(conn_client, NAMESPACE);

    let object = client.get(TEST_OBJECT).await?;

    client
        .delete(TEST_OBJECT, &DeleteParams::default())
        .await?
        .map_left(|_| {
            // Object deleting
        })
        .map_right(|_| {
            // Object deleted
        });

    Ok(object)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Started ephemeral-roles-operator");

    let conn_client = Client::try_default().await?;
    let resource_client: Api<controller::Version> = Api::all(conn_client.clone());
    let join_handle =
        tokio::spawn(async move { manage(conn_client, resource_client, resource_handler).await });

    join_handle.await?;

    Ok(())
}
