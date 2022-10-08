use anyhow::Context;
use arq_plugins::prelude::*;
use tracing::info;
use walkdir::WalkDir;

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();
    info!("Staring ARQ CORE");
    unsafe {
        dotenv::dotenv().ok();

        let mut manager = PluginManager::new();

        let plugin_location = std::env::var("PLUGIN_LOCATION")
            .context("PLUGIN_LOCATION not set")
            .unwrap();
        
        let ignore_corrupted_plugins = std::env::var("IGNORE_CORRUPTED_PLUGINS").unwrap_or_else(|_| "false".to_string());

        for entry in WalkDir::new("../plugins")
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                continue;
            }

            let component_results = manager.load_components(entry.path());
            let middleware_results = manager.load_middleware(entry.path());

            if ignore_corrupted_plugins == "true" && component_results.is_err() && middleware_results.is_err() {
                panic!("Corrupted library detected: {}", entry.path().display())
            }
        }

    }
}
