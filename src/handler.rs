use crate::{deployer, deployer::AsyncResult, ephemeral_roles};
use kube::{api::ResourceExt, runtime::watcher::Event, Client};

pub async fn handle(
    conn_client: Client,
    watch_event: Event<ephemeral_roles::ERVersion>,
) -> AsyncResult<()> {
    match watch_event {
        Event::Applied(er_version) => {
            let version = er_version.name();

            println!("Starting ERVersion {} rollout", version);
            deployer::deploy(conn_client, er_version).await?;
            println!("ERVersion {} rollout complete", version);
        }
        Event::Deleted(er_version) => {
            let version = er_version.name();

            println!("Starting ERVersion {} deletion", version);
            deployer::remove(conn_client, er_version).await?;
            println!("ERVersion {} deletion complete", version);
        }
        Event::Restarted(_) => {}
    }

    Ok(())
}
