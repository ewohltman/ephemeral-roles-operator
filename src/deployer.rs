use k8s_openapi::api::core::v1::ConfigMap;
use kube::core::params::{DeleteParams, PostParams};
use kube::core::ObjectMeta;
use kube::{api::Api, Client};

const TEST_OBJECT: &str = "test-object";
const NAMESPACE: &str = "ephemeral-roles";

pub async fn create_object(conn_client: Client) -> Result<ConfigMap, kube::Error> {
    let client: Api<ConfigMap> = Api::namespaced(conn_client, NAMESPACE);

    client
        .create(
            &PostParams::default(),
            &ConfigMap {
                metadata: ObjectMeta {
                    name: Some(TEST_OBJECT.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
}

pub async fn delete_object(conn_client: Client) -> Result<ConfigMap, kube::Error> {
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
