use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::Mutex;
use tracing::info;

use bnpm_provenance::{DefaultKeylessSigner, ProvenanceService};
use bnpm_registry::{ContentService, NameService, RegistryService};
use bnpm_store::cas::CasStore;

#[derive(Parser)]
#[command(name = "bnpm-gateway", about = "Bnpm DPR gRPC gateway server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0:9481")]
    addr: String,

    #[arg(long, default_value = "./bnpm-store")]
    store_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let store_path = PathBuf::from(&args.store_path);
    let store = CasStore::new(store_path).await?;

    let name_service = Arc::new(NameService::new());
    let content_service = Arc::new(ContentService::new(store));
    let signer = DefaultKeylessSigner::new();
    let provenance_service = Arc::new(Mutex::new(ProvenanceService::new(signer)));
    let registry_service = RegistryService::new(name_service, content_service, provenance_service);

    let addr: SocketAddr = args.addr.parse()?;
    info!("Starting Bnpm DPR gateway on {}", addr);

    bnpm_proto::run_server(addr, registry_service).await?;

    Ok(())
}
