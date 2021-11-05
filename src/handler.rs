use crate::ephemeral_roles;
use kube::{api::ResourceExt, runtime::watcher::Event, Client};

pub async fn handle(
    _conn_client: Client,
    watch_event: Event<ephemeral_roles::ERVersion>,
) -> Result<(), kube::Error> {
    match watch_event {
        Event::Applied(er_version) => {
            println!("ERVersion applied: {}", er_version.name());
            // deployer::deploy(conn_client, er_version.name().as_str()).await?
        }
        Event::Deleted(er_version) => {
            println!("ERVersion deleted: {}", er_version.name());
            // deployer::remove(conn_client, er_version.name().as_str()).await?
        }
        Event::Restarted(_) => {}
    }

    Ok(())
}
