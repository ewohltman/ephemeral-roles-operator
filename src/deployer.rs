use crate::ephemeral_roles;
use kube::core::params::{Patch, PatchParams, PostParams};
use kube::core::{ApiResource, DynamicObject, GroupVersion, GroupVersionKind, TypeMeta};
use kube::{Api, Client, Resource, ResourceExt};
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
    for component in er_version.spec.components.iter() {
        for file in component.files.iter() {
            let resp = reqwest::get(file).await?.text().await?;
            let object: DynamicObject = serde_yaml::from_str(resp.as_str())?;
            let client = api_client(conn_client.clone(), &object)?;

            apply(
                client,
                component.name.as_str(),
                component.version.as_str(),
                object,
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn remove(_conn_client: Client, _version: &str) -> Result<(), Box<dyn error::Error>> {
    Ok(())
}

fn api_client(
    conn_client: Client,
    object: &DynamicObject,
) -> Result<Api<DynamicObject>, Box<dyn error::Error>> {
    let meta: TypeMeta;

    match &object.types {
        Some(type_meta) => meta = type_meta.to_owned(),
        None => {
            return Err(Box::new(UnknownObject {
                object: object.to_owned(),
            }))
        }
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

async fn apply(
    client: Api<DynamicObject>,
    component_name: &str,
    component_version: &str,
    object: DynamicObject,
) -> Result<(), Box<dyn error::Error>> {
    println!(
        "Apply {} {}: {}",
        component_name,
        component_version,
        object.clone().types.unwrap().kind
    );

    if let Err(err) = create(&client, &object).await {
        match err {
            kube::Error::Api(err) => {
                if err.code == 409 {
                    update(&client, &object).await?;
                }
            }
            _ => return Err(Box::new(err)),
        }
    }

    Ok(())
}

async fn create(client: &Api<DynamicObject>, object: &DynamicObject) -> Result<(), kube::Error> {
    let params = &PostParams::default();
    client.create(params, object).await?;
    Ok(())
}

async fn update(client: &Api<DynamicObject>, object: &DynamicObject) -> Result<(), kube::Error> {
    let name = object
        .meta()
        .name
        .to_owned()
        .unwrap_or_else(|| "".to_string());
    let patch = serde_json::to_value(&object)?;
    let patch = Patch::Apply(&patch);
    let params = PatchParams::apply("ephemeral-roles-operator");

    client.patch(name.as_str(), &params, &patch).await?;

    Ok(())
}
