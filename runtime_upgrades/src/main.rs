mod extrinsics;
mod kilt;
mod migrations;

use subxt::OnlineClient;

use kilt::KiltConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<KiltConfig>::from_url("ws://localhost:40047").await?;

    #[cfg(feature = "pre-eth-migration")]
    migrations::eth::pre::spawn_linked_dids(api, 2).await?;

    Ok(())
}
