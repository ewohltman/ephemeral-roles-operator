use k8s_openapi::{
    api::{
        apps::v1::{StatefulSet, StatefulSetSpec},
        core::v1::{
            Container, ContainerPort, EnvVar, EnvVarSource, ObjectFieldSelector, PodSpec,
            PodTemplateSpec, ResourceRequirements, SecretKeySelector,
        },
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::LabelSelector},
};
use kube::{
    api::Api,
    core::params::{DeleteParams, PostParams},
    core::ObjectMeta,
    Client,
};
use std::{array::IntoIter, collections::BTreeMap, error};

const NAMESPACE: &str = "ephemeral-roles";

pub async fn deploy(conn_client: Client, version: &str) -> Result<(), Box<dyn error::Error>> {
    let _client: Api<StatefulSet> = Api::namespaced(conn_client, NAMESPACE);
    let _params = &PostParams::default();

    // client.create(params, &statefulset(version)).await?;

    Ok(())
}

pub async fn remove(conn_client: Client, _version: &str) -> Result<(), Box<dyn error::Error>> {
    let _client: Api<StatefulSet> = Api::namespaced(conn_client, NAMESPACE);
    let _params = &DeleteParams::default();

    // client.delete(TEST_OBJECT, params).await?;

    Ok(())
}

fn statefulset(version: &str) -> StatefulSet {
    let replicas = 10;

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "ephemeral-roles".to_string());

    StatefulSet {
        metadata: ObjectMeta {
            name: Some(TEST_OBJECT.to_string()),
            ..Default::default()
        },
        spec: Option::from(StatefulSetSpec {
            replicas: Option::from(replicas),
            selector: LabelSelector {
                match_labels: Option::from(labels.clone()),
                ..Default::default()
            },
            service_name: "ephemeral-roles".to_string(),
            template: PodTemplateSpec {
                metadata: Option::from(ObjectMeta {
                    labels: Option::from(labels),
                    ..Default::default()
                }),
                spec: Option::from(PodSpec {
                    containers: vec![
                        ephemeral_roles_container(version, replicas),
                        jaeger_agent_container(),
                    ],
                    termination_grace_period_seconds: Option::from(30),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn ephemeral_roles_container(version: &str, replicas: i32) -> Container {
    let resources: BTreeMap<String, Quantity> = BTreeMap::from_iter(IntoIter::new([(
        "memory".to_string(),
        Quantity("512Mi".to_string()),
    )]));

    Container {
        env: Option::from(vec![
            env_var(
                "SHARD_COUNT",
                EnvVarValue::Value(format!("{}", replicas).as_str()),
            ),
            env_var("LOG_LEVEL", EnvVarValue::Value("info")),
            env_var(
                "LOG_TIMEZONE_LOCATION",
                EnvVarValue::Value("America/New_York"),
            ),
            env_var(
                "JAEGER_SERVICE_NAME",
                EnvVarValue::Value("ephemeral-roles.ephemeral-roles"),
            ),
            env_var("JAEGER_PROPAGATION", EnvVarValue::Value("jaeger,b3")),
            env_var(
                "INSTANCE_NAME",
                EnvVarValue::Source(Box::new(EnvVarSource {
                    field_ref: Option::from(ObjectFieldSelector {
                        field_path: "metadata.name".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            ),
            env_var(
                "BOT_TOKEN",
                EnvVarValue::Source(Box::new(EnvVarSource {
                    secret_key_ref: Option::from(SecretKeySelector {
                        key: "bot-token".to_string(),
                        name: Option::from("ephemeral-roles".to_string()),
                        optional: None,
                    }),
                    ..Default::default()
                })),
            ),
            env_var(
                "DISCORDRUS_WEBHOOK_URL",
                EnvVarValue::Source(Box::new(EnvVarSource {
                    secret_key_ref: Option::from(SecretKeySelector {
                        key: "discordrus-webhook-url".to_string(),
                        name: Option::from("ephemeral-roles".to_string()),
                        optional: None,
                    }),
                    ..Default::default()
                })),
            ),
        ]),
        image: Option::from(format!("ewohltman/ephemeral-roles:{}", version)),
        image_pull_policy: Option::from("Always".to_string()),
        name: "ephemeral-roles".to_string(),
        ports: Option::from(vec![ContainerPort {
            container_port: 8081,
            name: Option::from("http".to_string()),
            ..Default::default()
        }]),
        resources: Option::from(ResourceRequirements {
            limits: Option::from(resources.clone()),
            requests: Option::from(resources),
        }),
        ..Default::default()
    }
}

fn jaeger_agent_container() -> Container {
    let resources: BTreeMap<String, Quantity> = BTreeMap::from_iter(IntoIter::new([
        ("cpu".to_string(), Quantity("100Mi".to_string())),
        ("memory".to_string(), Quantity("256Mi".to_string())),
    ]));

    Container {
        args: Option::from(vec![
            "--reporter.grpc.host-port=dns:///jaeger-collector-headless.ephemeral-roles:14250"
                .to_string(),
            "--reporter.type=grpc".to_string(),
        ]),
        env: Option::from(vec![
            env_var(
                "POD_NAME",
                EnvVarValue::Source(Box::new(EnvVarSource {
                    field_ref: Option::from(ObjectFieldSelector {
                        field_path: "metadata.name".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            ),
            env_var(
                "HOST_IP",
                EnvVarValue::Source(Box::new(EnvVarSource {
                    field_ref: Option::from(ObjectFieldSelector {
                        field_path: "status.hostIP".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            ),
        ]),
        image: Option::from("jaegertracing/jaeger-agent:1.17.1".to_string()),
        image_pull_policy: Option::from("IfNotPresent".to_string()),
        name: "ephemeral-roles".to_string(),
        ports: Option::from(vec![
            ContainerPort {
                container_port: 5775,
                name: Option::from("zk-compact-trft".to_string()),
                protocol: Option::from("UDP".to_string()),
                ..Default::default()
            },
            ContainerPort {
                container_port: 5778,
                name: Option::from("config-rest".to_string()),
                protocol: Option::from("TCP".to_string()),
                ..Default::default()
            },
            ContainerPort {
                container_port: 6831,
                name: Option::from("jg-compact-trft".to_string()),
                protocol: Option::from("UDP".to_string()),
                ..Default::default()
            },
            ContainerPort {
                container_port: 6832,
                name: Option::from("jg-binary-trft".to_string()),
                protocol: Option::from("UDP".to_string()),
                ..Default::default()
            },
            ContainerPort {
                container_port: 14271,
                name: Option::from("admin-http".to_string()),
                protocol: Option::from("TCP".to_string()),
                ..Default::default()
            },
        ]),
        resources: Option::from(ResourceRequirements {
            limits: Option::from(resources.clone()),
            requests: Option::from(resources),
        }),
        ..Default::default()
    }
}

enum EnvVarValue<'a> {
    Value(&'a str),
    Source(Box<EnvVarSource>),
}

fn env_var(name: &str, value: EnvVarValue) -> EnvVar {
    match value {
        EnvVarValue::Value(value) => EnvVar {
            name: name.to_string(),
            value: Option::from(value.to_string()),
            ..Default::default()
        },
        EnvVarValue::Source(source) => EnvVar {
            name: name.to_string(),
            value_from: Option::from(*source),
            ..Default::default()
        },
    }
}
