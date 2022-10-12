use anyhow::Context;
use arq_plugins::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, debug};
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

#[tokio::main]
async fn main() {
    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(false) // don't include levels in formatted output
        .with_target(false) // don't include targets
        .with_thread_ids(false) // include the thread ID of the current thread
        .with_thread_names(false) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.

    let filter = EnvFilter::new("debug");

    tracing_subscriber::fmt()
        .event_format(format)
        .with_env_filter(filter)
        .init();

    let opt = Opt::from_args();

    info!("Staring ARQ CORE");
    unsafe {
        dotenv::dotenv().ok();

        let mut manager = PluginManager::new();

        let plugin_location = std::env::var("PLUGIN_LOCATION")
            .context("PLUGIN_LOCATION not set")
            .unwrap_or_else(|_| "./plugins".to_string());
        
        debug!("Looking for plugins in {}", &plugin_location);

        let ignore_corrupted_plugins =
            std::env::var("IGNORE_CORRUPTED_PLUGINS").unwrap_or_else(|_| "false".to_string());

        for entry in WalkDir::new(plugin_location)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                continue;
            }

            let component_results = manager.load_components(entry.path());
            let middleware_results = manager.load_middleware(entry.path());

            if ignore_corrupted_plugins == "true"
                && component_results.is_err()
                && middleware_results.is_err()
            {
                panic!("Corrupted library detected: {}", entry.path().display())
            }
        }

        let routes = manager.get_routes();
        let middlewares = manager.get_middlewares();

        let mut core = rocket::build().mount("/", routes);

        for fairing in middlewares {
            core = core.attach(fairing);
        }

        let core_handle = core
            .launch()
            .await
            .context("Something went wrong!")
            .unwrap();
    }
}
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "i", long)]
    installer: bool,
}
