use crate::ephemeral_roles;
// use kube::core::object::HasSpec;
use kube::core::params::PostParams;
use kube::core::{ApiResource, DynamicObject, GroupVersion, GroupVersionKind, TypeMeta};
use kube::{Api, Client, ResourceExt};
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct UnknownObject {
    object: DynamicObject,
}

impl error::Error for UnknownObject {}

impl fmt::Display for UnknownObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown object: {:?}", self.object)
    }
}

pub async fn deploy(
    conn_client: Client,
    er_version: ephemeral_roles::ERVersion,
) -> Result<(), Box<dyn error::Error>> {
    for (_name, component) in er_version.spec.components.iter() {
        for file in component.files.iter() {
            let resp = reqwest::get(file).await?.text().await?;
            let object: DynamicObject = serde_yaml::from_str(resp.as_str())?;
            let client = api_client(conn_client.clone(), object.clone())?;
            let params = &PostParams::default();

            println!("Apply: {}", object.clone().types.unwrap().kind);

            if let Err(err) = client.create(params, &object).await {
                println!("Error: {:?}", err);
            }
        }
    }

    Ok(())
}

pub async fn remove(_conn_client: Client, _version: &str) -> Result<(), Box<dyn error::Error>> {
    /*let _client: Api<StatefulSet> = Api::namespaced(conn_client, NAMESPACE);
    let _params = &DeleteParams::default();*/

    // client.delete("ephemeral-roles".to_string(), params).await?;

    Ok(())
}

fn api_client(
    conn_client: Client,
    object: DynamicObject,
) -> Result<Api<DynamicObject>, Box<dyn error::Error>> {
    let meta: TypeMeta;

    match &object.types {
        Some(type_meta) => meta = type_meta.to_owned(),
        None => return Err(Box::new(UnknownObject { object })),
    }

    let gv = GroupVersion::from_str(meta.api_version.as_str())?;
    let gvk = GroupVersionKind {
        group: gv.group,
        version: gv.version,
        kind: meta.kind,
    };
    let resource = ApiResource::from_gvk(&gvk);
    let client: Api<DynamicObject>;

    match object.namespace() {
        Some(namespace) => {
            client = Api::namespaced_with(conn_client, namespace.as_str(), &resource)
        }
        None => client = Api::all_with(conn_client, &resource),
    }

    Ok(client)
}
