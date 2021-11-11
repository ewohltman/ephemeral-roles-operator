use crate::ephemeral_roles;
use kube::core::params::{Patch, PatchParams, PostParams};
use kube::core::{ApiResource, DynamicObject, GroupVersion, GroupVersionKind};
use kube::{Api, Client, Resource, ResourceExt};
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use tokio::task::JoinHandle;

pub type AsyncResult<T> = Result<T, AsyncError>;
pub type AsyncError = Box<dyn error::Error + Send + Sync>;

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

#[derive(Debug)]
pub struct JoinErrors {
    errors: Vec<AsyncError>,
}

impl error::Error for JoinErrors {}

impl fmt::Display for JoinErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = format!("");

        for (i, error_message) in self.errors.iter().enumerate() {
            if i == 0 {
                output.push_str(format!("{:?}", error_message).as_str());
                continue;
            }

            output.push_str(format!("{:?}: {:?}", output, error_message).as_str());
        }

        write!(f, "{:?}", output)
    }
}

pub async fn deploy(
    conn_client: Client,
    er_version: ephemeral_roles::ERVersion,
) -> AsyncResult<()> {
    let mut join_handles: Vec<JoinHandle<AsyncResult<()>>> = vec![];

    for component in er_version.spec.components.into_iter() {
        join_handles.push(tokio::spawn(deploy_component(
            conn_client.clone(),
            component,
        )));
    }

    let mut errors: Vec<AsyncError> = vec![];

    for join_handle in join_handles.into_iter() {
        if let Err(err) = join_handle.await? {
            errors.push(err);
        }
    }

    if !errors.is_empty() {
        return Err(Box::new(JoinErrors { errors }));
    }

    Ok(())
}

pub async fn deploy_component(
    conn_client: Client,
    component: ephemeral_roles::Component,
) -> AsyncResult<()> {
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

    Ok(())
}

pub async fn remove(
    _conn_client: Client,
    _er_version: ephemeral_roles::ERVersion,
) -> AsyncResult<()> {
    Ok(())
}

fn api_client(conn_client: Client, object: &DynamicObject) -> AsyncResult<Api<DynamicObject>> {
    let meta = match &object.types {
        Some(meta) => meta.to_owned(),
        None => {
            return Err(Box::new(UnknownObject {
                object: object.to_owned(),
            }))
        }
    };

    let group_version = GroupVersion::from_str(meta.api_version.as_str())?;
    let resource = ApiResource::from_gvk(&GroupVersionKind {
        group: group_version.group,
        version: group_version.version,
        kind: meta.kind,
    });
    let client: Api<DynamicObject> = match object.namespace() {
        Some(namespace) => Api::namespaced_with(conn_client, namespace.as_str(), &resource),
        None => Api::all_with(conn_client, &resource),
    };

    Ok(client)
}

async fn apply(
    client: Api<DynamicObject>,
    component_name: &str,
    component_version: &str,
    object: DynamicObject,
) -> AsyncResult<()> {
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
